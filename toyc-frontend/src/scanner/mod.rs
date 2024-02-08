use std::io::{BufRead, BufReader, Cursor, Lines, Read, Seek};
use std::iter::{Peekable, Scan};
use std::num::ParseFloatError;
use std::str::Chars;
use thiserror::Error;
use crate::scanner::token::{Keyword, Token};

pub mod token;

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

pub struct ToyCScanner<S: Read> {
    buffer: String,
    state: State,
    lines: Peekable<Lines<BufReader<S>>>,
    position: usize,
}

#[derive(Debug, Error, PartialEq)]
pub enum ScannerError{
    #[error("Illegal character {0}")]
    IllegalCharacter(char),

    #[error("Malformed number {0}")]
    MalformedNumber(String),
}

impl<S: Read> ToyCScanner<S>{

    pub fn new(stream: S) -> Self{
        Self{
            state: State::Initial,
            lines: BufReader::new(stream).lines().peekable(),
            buffer: String::new(),
            position: 0,
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
                                None
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
        if let Some(mut line) = self.lines.next(){
            match &mut line{
                Ok(data) => self.position = 0,
                Err(e) => panic!("io error"),
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
                        ('0'..='9') => self.change_state(State::Identifier,c),
                        _ => return Err(ScannerError::IllegalCharacter(c)) ,
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
                        ('0'..='9') => self.buffer.push(c),
                        'E' => self.change_state(State::Exponent,c),
                        '.' => self.change_state(State::Float,c),
                        _ => return match self.buffer.parse::<f64>(){
                            Ok(num) =>  Ok(Token::Number(num)),
                            Err(_) =>  Err(ScannerError::MalformedNumber(self.buffer.clone()))
                        }
                    }
                }
                State::Sign => {
                    match c{
                        '+' | '-' => self.change_state(State::Exponent,c),
                        _ => return Err(ScannerError::MalformedNumber(self.buffer.clone()))
                    }
                }

                State::Exponent =>{
                    match c{
                        ('0'..='9') => self.push_char(c),
                        _ => return match self.buffer.parse::<f64>(){
                            Ok(num) =>  Ok(Token::Number(num)),
                            Err(_) =>  Err(ScannerError::MalformedNumber(self.buffer.clone()))
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
        Ok(Token::Eof)
    }

}

fn keyword_or_id_token(data: &str) -> Token{
    match data{
        "int" => Token::Keyword(Keyword::Int),
        _ => Token::Identifier(data.to_owned()),
    }
}

#[test]
fn test_scanner(){
    let data = "int a\n int b";
    let mut scanner = ToyCScanner::new(data.as_bytes());
    let t = scanner.next_token();

    assert_eq!(t,Ok(Token::Keyword(Keyword::Int)))
}