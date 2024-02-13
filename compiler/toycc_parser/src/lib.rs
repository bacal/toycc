use std::io::{BufReader, Read, Seek};
use std::string::ParseError;
use toycc_report::{Diagnostic, Report, ReportLevel};
use crate::scanner::token::Token;
use crate::scanner::{Scanner, };
use crate::scanner::error::ScannerError;

mod scanner;

pub struct Parser<T: Read + Seek>{
    scanner: Scanner<T>,
}


impl<T: Read + Seek> Parser<T>{
    pub fn new(stream: T, stream_name: String, debug: Option<i32>) -> Self{
        Self{
            scanner: Scanner::new(stream,stream_name, debug)
        }
    }

    pub fn parse(&mut self)-> Result<(),ParserError>{
        self.scanner.next_token()?;
        Ok(())
    }

    fn advance(&mut self) -> Result<Token,ScannerError>{
        self.scanner.next_token()
    }
}

#[derive(Report)]
pub enum ParserError{
    ScannerError(ScannerError)
}

impl From<ScannerError> for ParserError{
    fn from(value: ScannerError) -> Self {
        Self::ScannerError(value)
    }
}

impl Diagnostic for ParserError{
    fn info(&self) -> String {
        match self{
            ParserError::ScannerError(s) => s.info()
        }
    }

    fn level(&self) -> ReportLevel {
        match self{
            ParserError::ScannerError(s) => s.level()
        }
    }

    fn help(&self) -> Option<&str> {
        match self{
            ParserError::ScannerError(s) => s.help()
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        match self{
            ParserError::ScannerError(s) => s.others()
        }
    }
}