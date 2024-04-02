mod ast;
pub mod error;

use crate::parser::ast::{Definition, FuncDef, Program, Statement, VarDef};
use crate::parser::error::{ParserError, ParserErrorKind};
use crate::scanner::error::ScannerError;
use crate::scanner::token::*;
use crate::scanner::Scanner;
use crate::BufferedStream;
use std::env::join_paths;
use std::io::{Read, Seek};

pub struct Parser<S: Read + Seek> {
    scanner: Scanner<S>,
    debug: Option<u32>,
    verbose: bool,
    rewind: bool,
    token: Token,
}

impl<'a, S: Read + Seek> Parser<S> {
    pub fn new(stream: S, file_name: &'a str, debug: Option<u32>, verbose: bool) -> Self {
        Self {
            scanner: Scanner::new(
                BufferedStream::new(stream, Some(file_name.to_string())),
                debug,
                verbose,
            ),
            debug,
            verbose,
            rewind: false,
            token: Token::new(TokenKind::Eof, 0),
        }
    }

    fn debug_print(&self, message: &str) {
        match self.debug.unwrap_or(4) {
            0 | 2 => println!("[PARSER] {message}"),
            _ => {}
        }
    }

    fn next_token(&mut self) -> Result<&Token, ScannerError> {
        match self.rewind {
            true => {
                self.rewind = false;
                Ok(&self.token)
            }
            false => {
                self.token = self.scanner.next_token()?;
                Ok(&self.token)
            }
        }
    }

    fn accept(
        &mut self,
        token_kind: TokenKind,
        error_kind: ParserErrorKind,
    ) -> Result<(), ParserError> {
        if self.next_token()?.kind != token_kind {
            Err(self.create_error(error_kind))
        } else {
            Ok(())
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut definitions = vec![];
        loop {
            match &self.next_token()?.kind {
                TokenKind::Eof => break,
                _ => {
                    self.rewind = true;
                    definitions.push(self.definition()?)
                }
            }
        }
        println!("{:#?}", definitions);
        Ok(Program::Definition(definitions))
    }

    fn definition(&mut self) -> Result<Definition, ParserError> {
        self.debug_print("entering definition");
        let tc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t,
            _ => return Err(self.create_error(ParserErrorKind::ExpectedType)),
        }
        .clone();

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => id,
            _ => return Err(self.create_error(ParserErrorKind::ExpectedIdentifier)),
        }
        .clone();

        let def = match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::Semicolon) => {
                Definition::VarDef(VarDef::new(vec![identifier], tc_type.to_string()))
            }
            TokenKind::Delimiter(Delimiter::LParen) => {
                self.rewind = true;
                let (vardefs, statement) = self.func_def()?;
                Definition::FuncDef(FuncDef::new(
                    identifier,
                    tc_type.to_string(),
                    vardefs,
                    statement,
                ))
            }
            _ => return Err(self.create_error(ParserErrorKind::ExpectedDelimiter('('))),
        };
        self.debug_print("exiting definition");
        Ok(def)
    }

    fn func_def(&mut self) -> Result<(Vec<VarDef>, Statement), ParserError> {
        self.debug_print("entering func_def");
        let header = self.func_header()?;
        let body = self.func_body()?;
        self.debug_print("exiting func_def");
        Ok((header, Statement::BlockState(vec![], vec![])))
    }
    fn func_header(&mut self) -> Result<Vec<VarDef>, ParserError> {
        self.debug_print("entering func_header");

        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter('('),
        )?;
        let params = self.formal_param_list()?;
        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(')'),
        )?;
        self.debug_print("exiting func_header");
        Ok(params)
    }
    fn formal_param_list(&mut self) -> Result<Vec<VarDef>, ParserError> {
        let mut param_list = vec![];
        self.debug_print("entering formal_param_list");
        let tc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t,
            TokenKind::Delimiter(Delimiter::RParen) => {
                self.rewind = true;
                return Ok(param_list);
            }
            _ => return Err(self.create_error(ParserErrorKind::ExpectedType)),
        }
        .clone();

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => id,
            _ => return Err(self.create_error(ParserErrorKind::ExpectedIdentifier)),
        }
        .clone();

        param_list.push(VarDef::new(vec![identifier], tc_type.to_string()));
        param_list.append(&mut self.rep_formal_param()?.unwrap_or_default());
        self.debug_print("exiting formal_param_list");
        Ok(param_list)
    }

    fn rep_formal_param(&mut self) -> Result<Option<Vec<VarDef>>, ParserError> {
        match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::Comma) => {}
            _ => {
                self.rewind = true;
                return Ok(None);
            }
        }
        let mut params = vec![];

        let tc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t,
            _ => return Err(self.create_error(ParserErrorKind::ExpectedType)),
        }
        .clone();

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => id,
            _ => return Err(self.create_error(ParserErrorKind::ExpectedIdentifier)),
        }
        .clone();
        params.push(VarDef::new(vec![identifier], tc_type.to_string()));
        if let Some(mut param) = self.rep_formal_param()?{
            params.append(&mut param)
        }
        Ok(Some(params))
    }

    fn func_body(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering func_body");

        let compound_statement = self.compound_statement()?;

        self.debug_print("exiting func_body");
        Ok(())
    }


    /// Todo: Finish compound statement implementation
    fn compound_statement(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering compound_statement");
        self.accept(
            TokenKind::Delimiter(Delimiter::LCurly),
            ParserErrorKind::ExpectedDelimiter('{'),
        )?;
        match &self.next_token()?.kind {
            TokenKind::Type(t) => {
                let tc_type = match &self.next_token()?.kind {
                    TokenKind::Type(Type::Int) | TokenKind::Type(Type::Char) => {}
                    _ => {ParserErrorKind::Generic;}
                };

            }
            _ => {
                let statements = self.statements()?;
            }
        }
        self.accept(
            TokenKind::Delimiter(Delimiter::RCurly),
            ParserErrorKind::ExpectedDelimiter('}'),
        )?;
        self.debug_print("exiting compound_statement");
        Ok(())
    }

    fn statements(&mut self) -> Result<(), ParserError> {
        // let statement = self.statement()?;
        // let statements = self.statements();
        Ok(())
    }

    /// Todo: finish if_statement production implementation.
    fn if_statement(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering if_statement");

        Ok(())
    }

    fn statement(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering statement");

        self.debug_print("exiting statement");

        Ok(())
    }


    fn null_statement(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering null statement");
        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(';')
        )?;
        
        self.debug_print("exiting null statement");

        Ok(())
    }

    pub fn return_statement(&mut self) -> Result<(),ParserError>{
        todo!()
    }
    pub fn while_statement(&mut self) -> Result<(),ParserError>{
        todo!()
    }
    pub fn read_statement(&mut self) -> Result<(),ParserError>{
        todo!()
    }
    pub fn write_statement(&mut self) -> Result<(),ParserError>{
        todo!()
    }
    pub fn new_line_statement(&mut self) -> Result<(),ParserError>{
        todo!()
    }
    pub fn expression_statement(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn break_statement(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn tidcs(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn stmtcs(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn else_stmt(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn ret_expr(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn read_rep(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn expression(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering expresssion");
        let relop_expression = self.relop_expression()?;
        let rep_expr = self.rep_expr()?;
        self.debug_print("exiting expression");
        Ok(())

    }
    pub fn rep_expr(&mut self) -> Result<(),ParserError>{
        self.debug_print("entering rep expr");
        match &self.next_token()?.kind {
            TokenKind::AssignOP => {}
            _ => {
                self.rewind = true;
                return Ok(());
            }
        }

        let relop_expression = self.relop_expression()?;
        self.debug_print("exiting rep expr");

        Ok(())
	}
    pub fn relop_expression(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering relop expression");
        let simple_expression = self.simple_expression()?;
        let rep_relop_expression = self.rep_relop_expr()?;
        self.debug_print("exiting relop expression");
        Ok(())
	}
    pub fn rep_relop_expr(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering rep_relop_expr");
        match &self.next_token()?.kind {
            TokenKind::RelOP(RelOP::GreaterThan) |
            TokenKind::RelOP(RelOP::GreaterEqual) |
            TokenKind::RelOP(RelOP::LessEqual) |
            TokenKind::RelOP(RelOP::LessThan) |
            TokenKind::RelOP(RelOP::EqualsEquals) |
            TokenKind::RelOP(RelOP::NotEquals) => {},

            _ => {
                self.rewind = true;
                return Ok(())
            }
        }

        let simple_expression = self.simple_expression()?;

        self.debug_print("exiting rep_relop_expr");
        Ok(())


	}
    pub fn simple_expression(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering simple_expression");
        let term = self.term()?;
        let rep_simple_expr= self.rep_simple_expr()?;
        self.debug_print("exiting simple_expression");
        Ok(())
	}
    pub fn rep_simple_expr(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering rep_simple_expr");
        match &self.next_token()?.kind {
            TokenKind::AddOP(AddOP::Plus) |
            TokenKind::AddOP(AddOP::Minus) |
            TokenKind::AddOP(AddOP::Or) => {},

            _ => {
                self.rewind = true;
                return Ok(())
            }
        }

        let term = self.term()?;

        self.debug_print("exiting rep_simple_expr");

        Ok(())
	}
    pub fn term(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering term");
        let primary = self.primary()?;
        let rep_term = self.rep_term()?;
        self.debug_print("exiting term");
        Ok(())
	}
    pub fn rep_term(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering rep_term");
        match &self.next_token()?.kind {
            TokenKind::MulOP(MulOP::Multiply) |
            TokenKind::MulOP(MulOP::Divide) |
            TokenKind::MulOP(MulOP::And) |
            TokenKind::MulOP(MulOP::Mod) => {},

            _ => {
                self.rewind = true;
                return Ok(())
            }
        }

        let primary = self.primary()?;

        self.debug_print("exiting rep_term");

        Ok(())
	}
    pub fn primary(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering primary");

        // match &self.next_token()?.kind {
        //     TokenKind::Identifier(id) => {
        //         let fcall_option = self.fcall_option()?;
        //         return Ok(())
        //     },

        //     TokenKind::Number => {
        //         self.accept(TokenKind::Number(num));
        //         Ok(())
        //     }

        //     _ => {Err(self.create_error(ParserErrorKind::Generic))}
        // }

        todo!()
	}
    pub fn fcall_option(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering fcall_option");
        match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::RParen) => {
                let function_call = self.function_call()?;
                Ok(())
            }

            _ => {
                self.rewind = true;
                Ok(())
            }
        }

	}
    pub fn not(&mut self) -> Result<(),ParserError>{
		todo!()
	}
    pub fn function_call(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering function_call");
        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter('(')
        )?;
        let aparam_option = self.aparam_option()?;
        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(')')
        )?;
        self.debug_print("exiting function_call");
        Ok(())
	}
    pub fn aparam_option(&mut self) -> Result<(),ParserError>{
		// match &self.next_token()?.kind {
        //     TokenKind::Identifier(id) |
        //     TokenKind::Number { num, sci = false} |
        //     TokenKind::String(string) |
        //     TokenKind::CharLiteral(ch) |
        //     TokenKind::Delimiter(Delimiter::LParen) |
        //     TokenKind::AddOP(AddOP::Minus) => {
        //         self.actual_parameters();
        //         Ok(())
        //     }

        //     _ => {
        //         self.rewind = true;
        //         Ok(())
        //     }
        // }

        todo!();
	}
    pub fn actual_parameters(&mut self) -> Result<(),ParserError>{
		self.debug_print("entering actual_parameters");
        let expression = self.expression()?;
        let rep_aparam_expr = self.rep_aparam_expr()?;
        self.debug_print("exiting actual_parameters");
        Ok(())
	}

    pub fn rep_aparam_expr(&mut self) -> Result<(),ParserError>{
		match &self.next_token()?.kind{
            TokenKind::Delimiter(Delimiter::Comma) => {}
            _ => {
                self.rewind = true;
                return Ok(())
            }
        }

        let expression = self.expression()?;
        Ok(())
	}

    fn create_error(&mut self, kind: ParserErrorKind) -> ParserError {
        let line = self.scanner.error_get_line(self.scanner.previous_location);
        let location = self.scanner.previous_location;
        let stream_name = self.scanner.stream.name.clone().unwrap_or_default();
        ParserError::new(kind, line, location, 1, stream_name, None)
    }
}
