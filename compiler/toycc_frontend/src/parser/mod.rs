mod ast;
pub mod error;

use crate::parser::ast::{Definition, FuncDef, Program, Statement, VarDef};
use crate::parser::error::ParserErrorKind::ExpectedIdentifier;
use crate::parser::error::{ParserError, ParserErrorKind};
use crate::scanner::error::ScannerError;
use crate::scanner::token::{Delimiter, Keyword, Token, TokenKind, Type};
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
        let mut declarations = vec![];

        let toyc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t.clone(),
            _ => {
                self.rewind = true;
                return Ok(declarations);
            }
        };

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => declarations.push((toyc_type, id.clone())),
            _ => return Err(self.create_error(ExpectedIdentifier)),
        };

        declarations.append(&mut self.declarations()?);

        Ok(declarations)
    }

    fn statements(&mut self) -> Result<(), Box<ParserError>> {
        Ok(())
    }

    /// Todo: finish if_statement production implementation.
    fn if_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering if_statement");

        Ok(())
    }

    fn try_statement(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }

    fn statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering statement");

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

    fn return_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering return_statement");
        match &self.next_token()?.kind {
            TokenKind::Keyword(Keyword::Return) => {}
            _ => return Err(self.create_error(ParserErrorKind::ExpectedKeyword(Keyword::Return))),
        };
        let expr = match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::Semicolon) => None,
            _ => Some(self.expression()?),
        };
        self.debug_print("exiting return_statement");
        Ok(())
    }
    fn while_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering while_statement");

        self.accept(
            TokenKind::Keyword(Keyword::While),
            ParserErrorKind::ExpectedKeyword(Keyword::While),
        )?;

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
            TokenKind::Keyword(Keyword::Read),
            ParserErrorKind::ExpectedKeyword(Keyword::Read),
        )?;

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
        todo!();
        self.debug_print("exiting write_statement");
    }
    fn new_line_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering new_line_statement");
        todo!();
        self.debug_print("exiting new_line_statement");
    }
    fn expression_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering expression_statement");
        todo!();
        self.debug_print("exiting expression_statement");
    }
    fn break_statement(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering break_statement");
        todo!();
        self.debug_print("exiting break_statement");
    }

    fn else_stmt(&mut self) -> Result<(), Box<ParserError>> {
        self.debug_print("entering else_stmt");
        todo!();
        self.debug_print("exiting else_stmt");
    }
    fn ret_expr(&mut self) -> Result<(), Box<ParserError>> {
        todo!();
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
        todo!()
    }
    fn rep_expr(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn relop_expression(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn rep_relop_expr(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn simple_expression(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn rep_simple_expr(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn term(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn rep_term(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn primary(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn fcall_option(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn not(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn function_call(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn aparam_option(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn actual_parameters(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }
    fn rep_aparam_expr(&mut self) -> Result<(), Box<ParserError>> {
        todo!()
    }

    fn create_error(&mut self, kind: ParserErrorKind) -> Box<ParserError> {
        let line = self.scanner.error_get_line(self.scanner.previous_location);
        let location = self.scanner.previous_location;
        let stream_name = self.scanner.stream.name.clone().unwrap_or_default();
        Box::new(ParserError::new(kind, line, location, 1, stream_name, None))
    }
}
