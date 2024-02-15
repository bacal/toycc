pub mod error;

use std::io::{Read, Seek};
use std::sync::Arc;
use crate::parser::error::ParserError;
use crate::scanner::error::ScannerError;
use crate::scanner::Scanner;
use crate::scanner::token::Token;


pub struct Parser<S: Read + Seek>
where Arc<S>: Read + Seek
{
    stream: Arc<S>,
    scanner: Scanner<S>,
}


impl<S: Read + Seek> Parser<S>
where Arc<S>: Read + Seek
{
    pub fn new(stream: Arc<S>, stream_name: String, debug: Option<u32>) -> Self{
        Self{
            stream: stream.clone(),
            scanner: Scanner::new(stream.clone(), stream_name, debug)
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
