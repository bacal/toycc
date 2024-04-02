mod ast;
pub mod error;

use crate::parser::ast::{Definition, FuncDef, Program, Statement, VarDef};
use crate::parser::error::{ParserError, ParserErrorKind};
use crate::scanner::error::ScannerError;
use crate::scanner::token::*;
use crate::scanner::Scanner;
use crate::BufferedStream;
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

    fn next_token(&mut self) -> Result<&Token, Box<ScannerError>> {
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
    ) -> Result<(), Box<ParserError>> {
        if self.next_token()?.kind != token_kind {
            Err(self.create_error(error_kind))
        } else {
            Ok(())
        }
    }

    pub fn parse(&mut self) -> Result<Program, Box<ParserError>> {
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

    fn definition(&mut self) -> Result<Definition, Box<ParserError>> {
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
            _ => {
                return Err(self.create_error(ParserErrorKind::ExpectedDelimiter(Delimiter::LParen)))
            }
        };
        self.debug_print("exiting definition");
        Ok(def)
    }

    fn func_def(&mut self) -> Result<(Vec<VarDef>, Statement), Box<ParserError>> {
        self.debug_print("entering func_def");
        let header = self.func_header()?;
        let body = self.func_body()?;
        self.debug_print("exiting func_def");
        Ok((header, Statement::BlockState(vec![], vec![])))
    }
    fn func_header(&mut self) -> Result<Vec<VarDef>, Box<ParserError>> {
        self.debug_print("entering func_header");

        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::LParen),
        )?;
        let params = self.formal_param_list()?;
        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::RParen),
        )?;
        self.debug_print("exiting func_header");
        Ok(params)
    }
    fn formal_param_list(&mut self) -> Result<Vec<VarDef>, Box<ParserError>> {
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

    fn rep_formal_param(&mut self) -> Result<Option<Vec<VarDef>>, Box<ParserError>> {
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
        if let Some(mut param) = self.rep_formal_param()? {
            params.append(&mut param)
        }
        Ok(Some(params))
    }

    fn func_body(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering func_body");

        let compound_statement = self.compound_statement()?;

        self.debug_print("exiting func_body");
        Ok(())
    }

    fn compound_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering compound_statement");
        self.accept(
            TokenKind::Delimiter(Delimiter::LCurly),
            ParserErrorKind::ExpectedDelimiter(Delimiter::LCurly),
        )?;
        let declarations = self.declarations()?;
        let stmts = self.statements()?;

        self.accept(
            TokenKind::Delimiter(Delimiter::RCurly),
            ParserErrorKind::ExpectedDelimiter(Delimiter::RCurly),
        )?;
        self.debug_print("exiting compound_statement");
        Ok(())
    }

    fn declarations(&mut self) -> Result<Vec<(Type, String)>, Box<ParserError>> {
        self.debug_print("entering declarations");
        let mut declarations = vec![];

        let toyc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t.clone(),
            _ => {
                self.rewind = true;
                self.debug_print("exiting declarations");
                return Ok(declarations);
            }
        };

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => declarations.push((toyc_type, id.clone())),
            _ => return Err(self.create_error(ParserErrorKind::ExpectedIdentifier)),
        };

        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(Delimiter::Semicolon),
        )?;

        declarations.append(&mut self.other_decls()?);

        self.debug_print("exiting declarations");

        Ok(declarations)
    }

    fn other_decls(&mut self) -> Result<Vec<(Type, String)>, Box<ParserError>> {
        let mut declarations = vec![];

        let toyc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t.clone(),
            _ => {
                println!("REWINDING");
                self.rewind = true;
                return Ok(declarations);
            }
        };

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => declarations.push((toyc_type, id.clone())),
            _ => return Err(self.create_error(ParserErrorKind::ExpectedIdentifier)),
        };

        declarations.append(&mut self.other_decls()?);

        Ok(declarations)
    }
    fn statements(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering statements");
        let x = match &self.next_token()?.kind {
            TokenKind::Keyword(Keyword::Break) => Some(self.break_statement()?),
            TokenKind::Delimiter(Delimiter::LCurly) => {
                self.rewind = true;
                Some(self.compound_statement()?)
            }
            TokenKind::Keyword(Keyword::If) => Some(self.if_statement()?),
            TokenKind::Delimiter(Delimiter::Semicolon) => Some(self.null_statement()?),
            TokenKind::Keyword(Keyword::Return) => self.return_statement()?,
            TokenKind::Keyword(Keyword::While) => Some(self.while_statement()?),
            TokenKind::Keyword(Keyword::Read) => Some(self.read_statement()?),
            TokenKind::Keyword(Keyword::Write) => Some(self.write_statement()?),
            TokenKind::Keyword(Keyword::Newline) => Some(self.new_line_statement()?),
            TokenKind::Identifier(_)
            | TokenKind::Number { .. }
            | TokenKind::String(_)
            | TokenKind::CharLiteral(_)
            | TokenKind::Delimiter(Delimiter::LParen)
            | TokenKind::Delimiter(Delimiter::Not)
            | TokenKind::AddOP(AddOP::Minus) => {
                self.rewind = true;
                Some(self.expression_statement()?)
            }
            _ => {
                self.rewind = true;
                self.debug_print("exiting statements");
                return Ok(None);
            }
        };
        self.statements()?;
        self.debug_print("exiting statements");
        Ok(x)
    }

    /// Todo: finish if_statement production implementation.
    fn if_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering if_statement");
        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::LParen),
        )?;
        let expression = self.expression();
        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::RParen),
        )?;
        let statement = self.statement();
        let toyc_else = self.else_stmt();

        Ok(())
    }

    fn statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering statement");

        let statement = match self.next_token()?.kind {
            TokenKind::Keyword(Keyword::Break) => self.break_statement(),
            TokenKind::Delimiter(Delimiter::LCurly) => {
                self.rewind = true;
                self.compound_statement()?;
                Ok(())
            }
            TokenKind::Keyword(Keyword::If) => self.if_statement(),
            TokenKind::Delimiter(Delimiter::Semicolon) => self.null_statement(),
            TokenKind::Keyword(Keyword::Return) => {
                self.return_statement();
                Ok(())
            }
            TokenKind::Keyword(Keyword::While) => self.while_statement(),
            TokenKind::Keyword(Keyword::Read) => self.read_statement(),
            TokenKind::Keyword(Keyword::Write) => self.write_statement(),
            TokenKind::Keyword(Keyword::Newline) => self.new_line_statement(),
            _ => {
                self.rewind = true;
                self.expression_statement()
            }
        };

        self.debug_print("exiting statement");

        Ok(())
    }

    fn null_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering null statement");

        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(Delimiter::Semicolon),
        )?;

        self.debug_print("exiting null statement");

        Ok(())
    }

    fn return_statement(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering return_statement");
        println!("{}", &self.next_token()?.kind);
        self.rewind = true;
        let expr = match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::Semicolon) => None,
            _ => {
                let expr = Some(self.expression()?);
                match &self.next_token()?.kind {
                    TokenKind::Delimiter(Delimiter::Semicolon) => {}
                    _ => {
                        return Err(self.create_error(ParserErrorKind::ExpectedDelimiter(
                            Delimiter::Semicolon,
                        )))
                    }
                }
                expr
            }
        };

        self.debug_print("exiting return_statement");
        Ok(expr)
    }
    fn while_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering while_statement");

        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::LParen),
        )?;

        let expr = self.expression()?;

        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::RParen),
        )?;

        let statement = self.statement()?;

        self.debug_print("exiting while_statement");
        Ok(())
    }
    fn read_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering read_statement");

        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::LParen),
        )?;

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => id,
            _ => return Err(self.create_error(ParserErrorKind::ExpectedIdentifier)),
        };

        let others = self.read_rep()?;

        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::RParen),
        )?;

        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(Delimiter::Semicolon),
        )?;

        self.debug_print("exiting read_statement");
        Ok(())
    }
    fn write_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering write_statement");

        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::LParen),
        )?;

        let params = self.actual_parameters()?;

        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::RParen),
        )?;

        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(Delimiter::Semicolon),
        )?;
        self.debug_print("exiting write_statement");
        Ok(())
    }
    fn new_line_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering new_line_statement");
        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(Delimiter::Semicolon),
        )?;
        self.debug_print("exiting new_line_statement");
        Ok(())
    }
    fn expression_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering expression_statement");
        let expression = self.expression();
        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(Delimiter::Semicolon),
        )?;
        self.debug_print("exiting expression_statement");
        Ok(())
    }
    fn break_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering break_statement");
        self.accept(
            TokenKind::Delimiter(Delimiter::Semicolon),
            ParserErrorKind::ExpectedDelimiter(Delimiter::Semicolon),
        )?;
        self.debug_print("exiting break_statement");
        Ok(())
    }

    fn else_stmt(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering else_stmt");
        let else_statement = match &self.next_token()?.kind {
            TokenKind::Keyword(Keyword::Else) => {
                let statement = self.statement()?;
                Some(())
            }
            _ => {
                self.rewind = true;
                None
            }
        };
        self.debug_print("exiting else_stmt");
        Ok(else_statement)
    }
    fn return_expr(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering return_expr");
        self.rewind = true;
        let a = if let Ok(expression) = self.expression() {
            Some(())
        } else {
            None
        };
        self.debug_print("exiting return_expr");
        Ok(a)
    }
    fn read_rep(&mut self) -> Result<Vec<String>, Box<ParserError>> {
        let mut repetitions = vec![];
        if self.next_token()?.kind == TokenKind::Delimiter(Delimiter::Comma) {
            let identifier = match &self.next_token()?.kind {
                TokenKind::Identifier(id) => repetitions.push(id.clone()),
                _ => return Err(self.create_error(ParserErrorKind::ExpectedIdentifier)),
            };
            repetitions.append(&mut self.read_rep()?);
        } else {
            self.rewind = true;
        }
        Ok(repetitions)
    }

    fn expression(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering expresssion");
        let relop_expression = self.relop_expression()?;
        let rep_expr = self.rep_expr()?;
        self.debug_print("exiting expression");
        Ok(())
    }
    fn rep_expr(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering rep_expr");

        let rep_expr = match &self.next_token()?.kind {
            TokenKind::AssignOP => {
                let relop = self.relop_expression()?;
                let x = self.rep_expr()?;
                Some(())
            }
            _ => {
                self.rewind = true;
                None
            }
        };

        self.debug_print("exiting rep_expr");
        Ok(rep_expr)
    }
    fn relop_expression(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering relop_expression");
        let expression = self.simple_expression()?;

        let relop_expression = self.rep_relop_expr()?;
        self.debug_print("exiting relop_expression");

        Ok(())
    }
    fn rep_relop_expr(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering rep_relop_expr");
        let relops = match &self.next_token()?.kind {
            TokenKind::RelOP(RelOP::GreaterThan)
            | TokenKind::RelOP(RelOP::GreaterEqual)
            | TokenKind::RelOP(RelOP::LessEqual)
            | TokenKind::RelOP(RelOP::LessThan)
            | TokenKind::RelOP(RelOP::EqualsEquals)
            | TokenKind::RelOP(RelOP::NotEquals) => {
                let simple_expression = self.simple_expression()?;
                let rep = self.rep_relop_expr()?;
                Ok(Some(()))
            }

            _ => {
                self.rewind = true;
                Ok(None)
            }
        };
        self.debug_print("exiting rep_relop_expr");
        relops
    }
    fn simple_expression(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering simple_expression");
        let term = self.term()?;
        let rep_simple_expr = self.rep_simple_expr()?;
        self.debug_print("exiting simple_expression");
        Ok(())
    }
    fn rep_simple_expr(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering rep_simple_expr");
        match &self.next_token()?.kind {
            TokenKind::AddOP(_) => {
                let term = self.term()?;
                let x = self.rep_simple_expr()?;
            }

            _ => {
                self.rewind = true;
                self.debug_print("exiting rep_simple_expr");
                return Ok(None);
            }
        }

        self.debug_print("exiting rep_simple_expr");

        Ok(Some(()))
    }
    fn term(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering term");
        let primary = self.primary()?;
        let rep_term = self.rep_term()?;
        self.debug_print("exiting term");
        Ok(())
    }
    fn rep_term(&mut self) -> Result<Option<()>, Box<ParserError>> {
        self.debug_print("entering rep_term");
        match &self.next_token()?.kind {
            TokenKind::MulOP(_) => {}

            _ => {
                self.rewind = true;
                self.debug_print("exiting rep_term");
                return Ok(None);
            }
        }

        let primary = self.primary()?;
        self.rep_term()?;
        self.debug_print("exiting rep_term");
        Ok(Some(()))
    }
    fn primary(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering primary");

        let primary = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => {
                let fcall_option = self.fcall_option()?;
                Ok(())
            }
            TokenKind::Number { num, .. } => Ok(()),
            TokenKind::String(s) => Ok(()),
            TokenKind::CharLiteral(c) => Ok(()),
            TokenKind::Delimiter(Delimiter::LParen) => {
                let expr = self.expression()?;
                self.accept(
                    TokenKind::Delimiter(Delimiter::RParen),
                    ParserErrorKind::ExpectedDelimiter(Delimiter::RParen),
                )?;
                Ok(())
            }
            TokenKind::Delimiter(Delimiter::Not) => {
                let primary = self.primary()?;
                Ok(())
            }
            _ => return Err(self.create_error(ParserErrorKind::Generic)),
        };
        primary
    }
    fn fcall_option(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering fcall_option");
        match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::LParen) => {
                self.rewind = true;
                let function_call = self.function_call()?;
                Ok(())
            }

            _ => {
                self.rewind = true;
                Ok(())
            }
        }
    }
    fn not(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn function_call(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering function_call");
        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::LParen),
        )?;
        let aparam_option = self.aparam_option()?;
        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserErrorKind::ExpectedDelimiter(Delimiter::RParen),
        )?;
        self.debug_print("exiting function_call");
        Ok(())
    }
    fn aparam_option(&mut self) -> Result<Option<()>, Box<ParserError>> {
        match &self.next_token()?.kind {
            TokenKind::Identifier(_)
            | TokenKind::Number { .. }
            | TokenKind::String(_)
            | TokenKind::CharLiteral(_)
            | TokenKind::Delimiter(_)
            | TokenKind::AddOP(_) => {
                let params = self.actual_parameters()?;
                Ok(Some(()))
            }

            _ => {
                self.rewind = true;
                Ok(None)
            }
        }
    }
    fn actual_parameters(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering actual_parameters");
        let expression = self.expression()?;
        let rep_aparam_expr = self.rep_aparam_expr()?;
        self.debug_print("exiting actual_parameters");
        Ok(())
    }
    fn rep_aparam_expr(&mut self) -> Result<(), Box<ParserError>> {
        match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::Comma) => {}
            _ => {
                self.rewind = true;
                return Ok(());
            }
        }

        let expression = self.expression()?;
        Ok(())
    }

    fn create_error(&mut self, kind: ParserErrorKind) -> Box<ParserError> {
        let line = self.scanner.error_get_line(self.scanner.previous_location);
        let location = self.scanner.previous_location;
        let stream_name = self.scanner.stream.name.clone().unwrap_or_default();
        Box::new(ParserError::new(kind, line, location, 1, stream_name, None))
    }
}
