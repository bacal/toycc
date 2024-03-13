pub mod error;
mod ast;

use crate::parser::error::ParserError;
use crate::scanner::error::ScannerError;
use crate::scanner::token::{Delimiter, Token, TokenKind};
use crate::scanner::Scanner;
use crate::BufferedStream;
use std::io::{Read, Seek};
use crate::parser::ast::{Definition, FuncDef, Statement, VarDef};

pub struct Parser<'a, S: Read + Seek> {
    scanner: Scanner<'a, S>,
    debug: Option<u32>,
    verbose: bool,
    rewind: bool,
    token: Token,
}

impl<'a, S: Read + Seek> Parser<'a, S> {
    pub fn new(stream: S, file_name: &'a str, debug: Option<u32>, verbose: bool) -> Self {
        Self {
            scanner: Scanner::new(BufferedStream::new(stream), file_name, debug, verbose),
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

    fn accept(&mut self, kind: TokenKind, error: ParserError) -> Result<(), ParserError> {
        if self.next_token()?.kind != kind {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        let d = self.definition()?;
        println!("{:#?}",d);
        Ok(())
    }

    fn definition(&mut self) -> Result<Definition, ParserError> {
        self.debug_print("entering definition");
        let tc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t,
            _ => return Err(ParserError::ExpectedType),
        }.clone();

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => id,
            _ => return Err(ParserError::ExpectedIdentifier),
        }.clone();

        let def = match &self.next_token()?.kind {
            TokenKind::Delimiter(Delimiter::Semicolon) => {
                Definition::VarDef(VarDef::new(vec![identifier],tc_type.to_string()))
            }
            TokenKind::Delimiter(Delimiter::LParen) => {
                self.rewind = true;
                let (vardefs,statement) = self.func_def()?;
                Definition::FuncDef(FuncDef::new(identifier,tc_type.to_string(),vardefs,statement))
            }
            _ => return Err(ParserError::ExpectedDelimiter('(')),
        };
        self.debug_print("exiting definition");
        Ok(def)
    }

    fn func_def(&mut self) -> Result<(Vec<VarDef>, Statement), ParserError> {
        self.debug_print("entering func_def");
        let header = self.func_header()?;
        let body = self.func_body()?;
        self.debug_print("exiting func_def");
        Ok((header,Statement::BlockState(vec![],vec![])))
    }
    fn func_header(&mut self) -> Result<Vec<VarDef>, ParserError> {
        self.debug_print("entering func_header");

        self.accept(
            TokenKind::Delimiter(Delimiter::LParen),
            ParserError::ExpectedDelimiter('('),
        )?;
        let params = self.formal_param_list()?;
        self.accept(
            TokenKind::Delimiter(Delimiter::RParen),
            ParserError::ExpectedDelimiter(')'),
        )?;
        self.debug_print("exiting func_header");
        Ok(params)
    }
    fn formal_param_list(&mut self) -> Result<Vec<VarDef>, ParserError> {
        let mut param_list = vec![];
        self.debug_print("entering formal_param_list");
        let tc_type = match &self.next_token()?.kind {
            TokenKind::Type(t) => t,
            _ => return Err(ParserError::ExpectedType),
        }.clone();

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => id,
            _ => return Err(ParserError::ExpectedIdentifier),
        }.clone();

        param_list.push(VarDef::new(vec![identifier],tc_type.to_string()));
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
            _ => return Err(ParserError::ExpectedType),
        }.clone();

        let identifier = match &self.next_token()?.kind {
            TokenKind::Identifier(id) => id,
            _ => return Err(ParserError::ExpectedIdentifier),
        }.clone();
        params.push(VarDef::new(vec![identifier],tc_type.to_string()));
        match self.rep_formal_param()?{
            Some(mut p) => params.append(&mut p),
            None => {}
        };
        Ok(Some(params))
    }

    fn func_body(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering func_body");

        let compound_statement = self.compound_statement()?;

        self.debug_print("exiting func_body");
        Ok(())
    }

    fn compound_statement(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering compound_statement");
        self.accept(
            TokenKind::Delimiter(Delimiter::LCurly),
            ParserError::ExpectedDelimiter('{'),
        )?;
        match &self.next_token()?.kind {
            TokenKind::Type(t) => {
                let tc_type = t;
            }
            _ => {
                let statements = self.statements()?;
            }
        }
        self.accept(
            TokenKind::Delimiter(Delimiter::RCurly),
            ParserError::ExpectedDelimiter('}'),
        )?;
        self.debug_print("exiting compound_statement");
        Ok(())
    }

    fn statements(&mut self) -> Result<(), ParserError> {
        // let statement = self.statement()?;
        // let statements = self.statements();
        Ok(())
    }

    fn statement(&mut self) -> Result<(), ParserError> {
        self.debug_print("entering statement");

        self.debug_print("exiting statement");

        Ok(())
    }
}
