pub mod error;

use crate::parser::error::ParserError;
use crate::scanner::token::TokenKind;
use crate::scanner::Scanner;
use crate::BufferedStream;
use std::io::{Read, Seek};

pub struct Parser<'a, S: Read + Seek> {
    scanner: Scanner<'a, S>,
    debug: Option<u32>,
    verbose: bool,
}

impl<'a, S: Read + Seek> Parser<'a, S> {
    pub fn new(stream: S, file_name: &'a str, debug: Option<u32>, verbose: bool) -> Self {
        Self {
            scanner: Scanner::new(BufferedStream::new(stream), file_name, debug, verbose),
            debug,
            verbose,
        }
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        let mut tokens_read = 0;
        loop {
            if self.scanner.next_token()?.kind == TokenKind::Eof {
                tokens_read += 1;
                break;
            }
            tokens_read += 1;
        }
        if self.debug.is_some() || self.verbose {
            println!("[SCANNER] Total tokens: {tokens_read}");
        }
        Ok(())
    }
}
