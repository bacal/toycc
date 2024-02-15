pub mod error;

use std::io::{Read, Seek};
use crate::parser::error::ParserError;
use crate::scanner::error::ScannerError;
use crate::scanner::Scanner;
use crate::scanner::token::Token;


pub struct Parser<'a, S: Read + Seek + 'a>
where &'a S: Read + Seek{
    stream: &'a S,
    scanner: Scanner<'a, S>,
}


impl<'a, S: Read + Seek> Parser<'a,S>
where &'a S: Read + Seek
{
    pub fn new(stream: &'a S, stream_name: String, debug: Option<u32>) -> Self{
        Self{
            stream,
            scanner: Scanner::new(stream, stream_name, debug)
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
