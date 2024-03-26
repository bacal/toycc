mod ast;
pub mod error;

use crate::parser::ast::{Definition, FuncDef, Program, Statement, VarDef};
use crate::parser::error::{ParserError, ParserErrorKind};
use crate::scanner::error::ScannerError;
use crate::scanner::token::{Delimiter, Token, TokenKind, Keyword, Type};
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
    fn create_error(&mut self, kind: ParserErrorKind) -> ParserError {
        let line = self.scanner.error_get_line(self.scanner.previous_location);
        let location = self.scanner.previous_location;
        let stream_name = self.scanner.stream.name.clone().unwrap_or_default();
        ParserError::new(kind, line, location, 1, stream_name, None)
    }
}
