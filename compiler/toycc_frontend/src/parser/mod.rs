pub mod error;

use std::io::{Read, Seek};
use std::ops::Deref;
use crate::BufferedStream;
use crate::parser::error::ParserError;
use crate::scanner::error::ScannerError;
use crate::scanner::Scanner;
use crate::scanner::token::Token;

pub struct Parser<'a, S: Read + Seek>{
    scanner: Scanner<'a, S>,
    debug: Option<u32>
}


impl<'a, S: Read + Seek> Parser<'a, S>
{
    pub fn new(stream: S, file_name: &'a str, debug: Option<u32>) -> Self{
        Self{
            scanner: Scanner::new(BufferedStream::new(stream),file_name,debug),
            debug
        }
    }

    pub fn parse(&mut self)-> Result<(),ParserError>{
        self.scanner.next_token()?;
        self.scanner.next_token()?;
        Ok(())
    }

    fn advance(&mut self) -> Result<Token,ScannerError>{
        self.scanner.next_token()
    }
}
