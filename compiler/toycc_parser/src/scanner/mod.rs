use std::io::{BufRead, BufReader, Lines, Read, Seek};
use std::iter::{Peekable};
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

pub struct Scanner<S: Read> {
    stream_name: String,
    buffer: String,
    state: State,
    previous_line: String,
    lines: Peekable<Lines<BufReader<S>>>,
    position: usize,
    debug: Option<i32>,
    lines_read: usize,
}

impl<S: Read> Scanner<S>{

    pub fn new(stream: S, stream_name: String, debug: Option<i32>) -> Self{
        Self{
            debug,
            state: State::Initial,
            lines: BufReader::new(stream).lines().peekable(),
            buffer: String::new(),
            position: 0,
            lines_read: 0,
            stream_name,
            previous_line: String::new(),
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
                                match self.get_char(){
                                    Some(c) => Some(c),
                                    None => Some(' '),
                                }
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
        self.previous_line = self.lines.peek().unwrap().as_ref().unwrap().clone();
        if let Some(mut line) = self.lines.next(){
            match &mut line{
                Ok(data) => {
                    if self.lines_read !=0{
                        self.position = 0;
                    }
                    self.lines_read+=1;
                }
                Err(_) => panic!("io error"),
            }
        }
    }
    fn change_state(&mut self, state: State, c: char){
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
                        _ => return Err(self.create_error(ScannerErrorKind::IllegalCharacter(c),0,None)),
                    }
                }

                State::Identifier => {
                    match c{
                        ('a'..='z') | ('A'..='Z') | ('0'..='9') => self.push_char(c),
                        _ => return Ok(keyword_or_id_token(self.buffer.as_str()))
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


                _ => {}
            }
        }
        // When we run out of data in our source stream we return the EOF token
        Ok(Token::new(TokenKind::Eof,0))
    }

    pub fn create_token(&mut self, kind: TokenKind, len: usize) -> Token{
        let token = Token::new(kind,len);
        if let Some(level) = self.debug{
            match level{
                _ => println!("{token}")
            }
        }
        token
    }
    fn create_error(&mut self, kind: ScannerErrorKind, len: usize, help: Option<String>) -> ScannerError{
        let line = self.lines.peek().unwrap_or(&Ok(self.previous_line.clone())).as_ref().unwrap().clone();
        ScannerError::new(kind,line,(self.lines_read,self.position+1),len,self.stream_name.clone(),help)
    }
}

fn keyword_or_id_token(data: &str) -> Token{
    let kind = match data{
        "int" => TokenKind::Keyword(Keyword::Int),
        _ => TokenKind::Identifier(data.to_owned()),
    };
    Token::new(kind, data.len())
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

