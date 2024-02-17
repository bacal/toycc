use std::io::{BufRead, BufReader, Lines, Read, Seek};
use std::iter::{Peekable};
use std::sync::Arc;
use crate::BufferedStream;
use crate::scanner::error::{ScannerError, ScannerErrorKind};
use crate::scanner::token::{Keyword, Token, TokenKind};

pub mod token;
pub mod error;

enum State{
    Initial,
    Identifier,
    Integer,
    Exponent,
    Sign,
    Float,
    Mulop,
    Addop,
    Assign,
}

pub struct Scanner<'a, S: Read + Seek>
{
    stream_name: &'a str,
    pub stream: BufferedStream<S>,
    buffer: String,
    state: State,
    position: usize,
    debug: Option<u32>,
    lines_read: usize,
    comments_nested: usize,
    previous_location: (usize, usize),
}

impl<'a, S: Read + Seek> Scanner<'a, S>
{

    pub fn new(stream: BufferedStream<S>, stream_name: &'a str, debug: Option<u32>) -> Self{
        Self{
            debug,
            stream,
            state: State::Initial,
            buffer: String::new(),
            lines_read: 0,
            comments_nested: 0,
            stream_name,
            position: 0,
            previous_location: (0, 0),
        }
    }
    pub fn get_char(&mut self) -> Option<char> {
        // Return next character in line
        // Buffer new if line is empty
        match &mut self.stream.peek(){
            Some(line) =>{
                match line.chars().nth(self.position){
                    Some(c) => Some(c),
                    None => {
                        self.next_line();
                        Some('\n')
                    }
                }
            },
            None => None,
        }
    }

    fn next_line(&mut self){
        let prev_position = self.position;
        if self.stream.next().is_some(){
            self.lines_read+=1;
            self.position =0;
        }
        if self.stream.peek().is_none(){
            self.position = prev_position;
            self.lines_read -=1;
        }
    }

    fn change_state(&mut self, state: State, c: char){
        self.previous_location = (self.lines_read,self.position+1);
        self.push_char(c);
        self.state = state;
    }


    fn push_char(&mut self, c: char){
        self.buffer.push(c);
        self.position+=1;
    }

    pub fn next_token(&mut self) -> Result<Token, ScannerError>{
        self.buffer.clear();
        while let Some(c) = self.get_char() {
            match self.state {
                State::Initial => {
                    match c {
                        ('a'..='z') | ('A'..='Z') => self.change_state(State::Identifier,c),
                        ('0'..='9') => self.change_state(State::Integer,c),
                        '\n' => {},
                        _ => return Err(self.create_error(ScannerErrorKind::IllegalCharacter(c),0,None)),
                    }
                }

                State::Identifier => {
                    match c{
                        ('a'..='z') | ('A'..='Z') | ('0'..='9') => self.push_char(c),
                        _ => return Ok(self.keyword_or_id_token())
                    }
                }

                State::Integer => {
                    match c{
                        ('0'..='9') => self.push_char(c),
                        'E' => self.change_state(State::Sign,c),
                        '.' => self.change_state(State::Float,c),
                        _ => return match self.buffer.parse::<f64>(){
                            Ok(num) =>  Ok(self.create_token(TokenKind::Number(num),self.buffer.len())),
                            Err(_) =>  Err(self.create_error(ScannerErrorKind::MalformedNumber(format!("invalid number {}",self.buffer)),1, None))
                        }
                    }
                }
                State::Sign => {
                    match c{
                        '+' | '-' => self.change_state(State::Exponent,c),
                        _ => return  Err(self.create_error(
                            ScannerErrorKind::MalformedNumber("exponent missing sign".to_string()),
                            1, Some("expected + or -".to_string())))
                    }
                }

                State::Exponent =>{
                    match c{
                        ('0'..='9') => self.push_char(c),
                        _ => return match self.buffer.parse::<f64>(){
                            Ok(num) =>  Ok(self.create_token(TokenKind::Number(num),self.buffer.len())),
                            Err(_) =>   Err(self.create_error(ScannerErrorKind::MalformedNumber("exponent has no digits".to_string()),1, None))
                        }
                    }
                }

                State::Float =>{
                    // match c{
                    //
                    // }
                }

                // State::CommentStart =>{
                //     match c{
                //         '/' => {
                //             self.next_line();
                //             self.state = State::Initial;
                //         }
                //         '*' => self.change_state(State::MultiLineEat),
                //         _ =>
                //     }
                // }

                _ => {}
            }
        }
        // When we run out of data in our source stream we return the EOF token
        Ok(Token::new(TokenKind::Eof,0,(self.lines_read,self.position)))
    }

    pub fn create_token(&mut self, kind: TokenKind, len: usize) -> Token{
        let token = Token::new(kind,len,(self.lines_read,self.position));
        if self.debug.is_some(){
            println!("[SCANNER] {token}")
        }
        self.previous_location = (self.lines_read, self.position+1);
        self.state = State::Initial;
        self.position+=1;
        token
    }
    fn create_error(&mut self, kind: ScannerErrorKind, len: usize, help: Option<String>) -> ScannerError{
        let location = (self.previous_location.0,self.previous_location.1+1);
        let line = match kind{
            ScannerErrorKind::MalformedNumber(_) =>{
                self.stream.rewind();
                self.stream.nth(location.0-1).unwrap()
            }
            _ => "".to_string()
        };
        ScannerError::new(kind,line,location,len,self.stream_name.to_string(),help)
    }
    fn keyword_or_id_token(&mut self) -> Token{
        let kind = match self.buffer.as_str(){
            "int" => TokenKind::Keyword(Keyword::Int),
            id => TokenKind::Identifier(id.to_string()),
        };
        self.create_token(kind,self.buffer.len())
    }
}


#[cfg(test)]
mod test_integration{
    use std::io::Cursor;
    use crate::BufferedStream;
    use super::Scanner;
    #[test]
    fn test_scanner() {
        let data = "232E1 a\n int b";
        let mut scanner = Scanner::new(BufferedStream::new(Cursor::new(data.as_bytes())), "name.tc",None);
        let t = scanner.next_token();

        println!("{}",t.err().unwrap());

        // assert_eq!(t, Ok(Token::new(TokenKind::Keyword(Keyword::Int),3)))
    }
}

