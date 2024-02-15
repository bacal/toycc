use std::io::{BufRead, BufReader, Lines, Read, Seek};
use std::iter::{Peekable};
use std::sync::Arc;
use toycc_report::{Diagnostic, Report};
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

pub struct Scanner<S>
where Arc<S>: Read + Seek,
{
    stream_name: String,
    stream: Arc<S>,
    buffer: String,
    state: State,
    lines: Peekable<Lines<BufReader<Arc<S>>>>,
    position: usize,
    debug: Option<u32>,
    lines_read: usize,
    comments_nested: usize,
    previous_location: (usize, usize),
}

impl<S> Scanner<S>
where Arc<S>: Read + Seek
{

    pub fn new(stream: Arc<S>, stream_name: String, debug: Option<u32>) -> Self{
        Self{
            debug,
            stream: stream.clone(),
            state: State::Initial,
            lines: BufReader::new(stream).lines().peekable(),
            buffer: String::new(),
            position: 0,
            lines_read: 0,
            stream_name,
            comments_nested: 0,
            previous_location: (0, 0),
        }
    }
    pub fn get_char(&mut self) -> Option<char> {
        // Return next character in line
        // Buffer new if line is empty
        match &mut self.lines.peek(){
            Some(line) =>{
                match line{
                    Ok(line) => {
                        match line.chars().nth(self.position){
                            Some(c) => Some(c),
                            None => {
                                self.next_line();
                                Some('\n')
                            }
                        }
                    }
                    Err(_) => panic!("io error")
                }
            },
            None => None,
        }
    }
    fn next_line(&mut self){
        let prev_position = self.position;
        if let Some(mut line) = self.lines.next(){
            match &mut line{
                Ok(data) => {
                    self.position = 0;
                    self.lines_read+=1;
                }
                Err(_) => panic!("io error"),
            }
        }
        if self.lines.peek().is_none(){
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
                        '\n' => {}
                        _ => return Err(self.create_error(ScannerErrorKind::IllegalCharacter(c),0,None)),
                    }
                }

                State::Identifier => {
                    match c{
                        ('a'..='z') | ('A'..='Z') | ('0'..='9') => self.push_char(c),
                        _ => return Ok(self.keyword_or_id_token(self.buffer.as_str()))
                    }
                }

                State::Integer => {
                    match c{
                        ('0'..='9') => self.push_char(c),
                        'E' => self.change_state(State::Sign,c),
                        '.' => self.change_state(State::Float,c),
                        _ => return match self.buffer.parse::<f64>(){
                            Ok(num) =>  Ok(self.create_token(TokenKind::Number(num),self.buffer.len())),
                            Err(_) =>  Err(self.create_error(ScannerErrorKind::MalformedNumber,1, None))
                        }
                    }
                }
                State::Sign => {
                    match c{
                        '+' | '-' => self.change_state(State::Exponent,c),
                        _ => return  Err(self.create_error(ScannerErrorKind::MalformedNumber,1, Some("expected + or -".to_string())))
                    }
                }

                State::Exponent =>{
                    match c{
                        ('0'..='9') => self.push_char(c),
                        _ => return match self.buffer.parse::<f64>(){
                            Ok(num) =>  Ok(self.create_token(TokenKind::Number(num),self.buffer.len())),
                            Err(_) =>   Err(self.create_error(ScannerErrorKind::MalformedNumber,1, Some("expected digit".to_string())))
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
        if let Some(level) = self.debug{
            match level{
                _ => println!("[SCANNER] {token}")
            }
        }
        self.previous_location = (self.lines_read, self.position+1);
        self.state = State::Initial;
        token
    }
    fn create_error(&mut self, kind: ScannerErrorKind, len: usize, help: Option<String>) -> ScannerError{
        let line = match kind{
            ScannerErrorKind::MalformedNumber =>{
                self.stream.rewind().unwrap();
                BufReader::new(self.stream.clone()).lines().nth(self.previous_location.0).unwrap().unwrap()
            }
            _ => "".to_string()
        };
        ScannerError::new(kind,line,self.previous_location,len,self.stream_name.clone(),help)
    }
    fn keyword_or_id_token(&self, data: &str) -> Token{
        let kind = match data{
            "int" => TokenKind::Keyword(Keyword::Int),
            _ => TokenKind::Identifier(data.to_owned()),
        };
        Token::new(kind, data.len(),(self.lines_read,self.position))
    }
}


#[cfg(test)]
mod test_integration{
    use super::*;
    #[test]
    fn test_scanner() {
        let data = "232E1 a\n int b";
        let mut scanner = Scanner::new(data.as_bytes(), "name.tc".to_string(),None);
        let t = scanner.next_token();

        println!("{}",t.err().unwrap());

        // assert_eq!(t, Ok(Token::new(TokenKind::Keyword(Keyword::Int),3)))
    }
}

