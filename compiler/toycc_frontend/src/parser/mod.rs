pub mod error;

use std::io::{Read, Seek};
use crate::parser::error::ParserError;
use crate::scanner::error::ScannerError;
use crate::scanner::Scanner;
use crate::scanner::token::Token;


pub struct Parser<T: Read + Seek>{
    scanner: Scanner<T>,
}


impl<T: Read + Seek> Parser<T>{
    pub fn new(stream: T, stream_name: String, debug: Option<u32>) -> Self{
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
