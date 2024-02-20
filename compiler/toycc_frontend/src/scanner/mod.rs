use crate::scanner::error::{ScannerError, ScannerErrorKind};
use crate::scanner::token::{Delimiter, Keyword, RelOP, Token, TokenKind};
use crate::BufferedStream;
use std::io::{Read, Seek};

use self::token::AddOP;
use self::token::MulOP;

pub mod error;
pub mod token;

enum State {
    Initial,
    Identifier,
    Integer,
    Scientific,
    Sign,
    Float,
    And,
    Or,
    Relationship,
    CommentStart,
    CommentEnd,
    CommentEat,
    String,
    CommentNested,
    CharLiteral,
    FloatFirst,
    SciFirst,
}

struct Placeholder;

pub struct Scanner<'a, S: Read + Seek> {
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

impl<'a, S: Read + Seek> Scanner<'a, S> {
    pub fn new(stream: BufferedStream<S>, stream_name: &'a str, debug: Option<u32>) -> Self {
        Self {
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
        match &mut self.stream.peek() {
            Some(line) => match line.chars().nth(self.position) {
                Some(c) => Some(c),
                None => {
                    self.next_line();
                    match self.get_char() {
                        Some(c) => Some(c),
                        None => Some('\n'),
                    }
                }
            },
            None => None,
        }
    }

    fn next_line(&mut self) {
        self.previous_location = (self.lines_read, self.position + 1);
        self.stream.next();
        self.lines_read += 1;
        self.position = 0;
    }

    fn change_state(&mut self, state: State, c: char) {
        self.previous_location = (self.lines_read, self.position + 1);
        self.push_char(c);
        self.state = state;
    }

    fn push_char(&mut self, c: char) {
        self.buffer.push(c);
        self.position += 1;
    }

    pub fn next_token(&mut self) -> Result<Token, ScannerError> {
        while let Some(c) = self.get_char() {
            match self.state {
                State::Initial => {
                    self.buffer.clear();
                    match c {
                        ('a'..='z') | ('A'..='Z') => self.change_state(State::Identifier, c),
                        ('0'..='9') => self.change_state(State::Integer, c),
                        ' ' | '\n' | '\t' => self.position += 1,
                        '<' | '>' | '!' | '=' => self.change_state(State::Relationship, c),
                        '*' => return Ok(self.create_token(TokenKind::MulOP(MulOP::Multiply), 1)),
                        '%' => return Ok(self.create_token(TokenKind::MulOP(MulOP::Mod), 1)),
                        '/' => self.change_state(State::CommentStart, c),
                        '&' => self.change_state(State::And, c),
                        '+' => return Ok(self.create_token(TokenKind::AddOP(AddOP::Plus), 1)),
                        '-' => return Ok(self.create_token(TokenKind::AddOP(AddOP::Minus), 1)),
                        '|' => self.change_state(State::Or, c),
                        '"' => self.change_state(State::String, c),
                        '\'' => self.change_state(State::CharLiteral, c),
                        ';' => return Ok(self.create_token(TokenKind::Delimiter(Delimiter::Semicolon),1)),
                        _ => {
                            return Err(self.create_error(
                                ScannerErrorKind::IllegalCharacter(c),
                                0,
                                None,
                            ))
                        }
                    }
                }

                State::Identifier => match c {
                    ('a'..='z') | ('A'..='Z') | ('0'..='9') => self.push_char(c),
                    _ => return Ok(self.keyword_or_id_token()),
                },

                State::Integer => match c {
                    ('0'..='9') => self.push_char(c),
                    'E' => self.change_state(State::Sign, c),
                    '.' => self.change_state(State::FloatFirst, c),
                    _ => {
                        return match self.buffer.parse::<f64>() {
                            Ok(num) => {
                                if c != '\n' {
                                    self.position -= 1;
                                }
                                Ok(self.create_token(TokenKind::Number(num), self.buffer.len()))
                            }
                            Err(_) => Err(self.create_error(
                                ScannerErrorKind::MalformedNumber(format!(
                                    "invalid number {}",
                                    self.buffer
                                )),
                                1,
                                None,
                            )),
                        }
                    }
                },
                State::Sign => match c {
                    '+' | '-' => self.change_state(State::SciFirst, c),
                    _ => {
                        return Err(self.create_error(
                            ScannerErrorKind::MalformedNumber("scientific number missing sign".to_string()),
                            1,
                            Some("expected + or -".to_string()),
                        ))
                    }
                },

                State::FloatFirst => {
                    match c{
                        ('0'..='9') => self.change_state(State::Float, c),
                        _ => return Err(self.create_error(
                            ScannerErrorKind::MalformedNumber("missing digits after decimal".to_string()),
                            self.buffer.len(),
                            Some("expected digit".to_string()))),
                    }
                }

                State::SciFirst => {
                    match c{
                        ('0'..='9') => self.change_state(State::Scientific, c),
                        _ => return Err(self.create_error(
                            ScannerErrorKind::MalformedNumber("missing digits after sign".to_string()),
                            self.buffer.len(),
                            Some("expected digit".to_string()))),
                    }
                }
                State::Float |
                State::Scientific => match c {
                    ('0'..='9') => self.push_char(c),
                    _ => return Ok(self.create_token(
                        TokenKind::Number(self.buffer.parse::<f64>().unwrap()),
                        self.buffer.len()))
                },

                State::CommentStart => match c {
                    '/' => {
                        self.next_line();
                        self.state = State::Initial;
                    }
                    '*' => {
                        self.change_state(State::CommentEat, c);
                        self.comments_nested += 1;
                    }
                    _ => return Ok(self.create_token(TokenKind::MulOP(MulOP::Divide), 1)),
                },

                State::CommentNested => {
                    if c == '*' {
                        self.comments_nested += 1;
                    }
                    self.state = State::CommentEat;
                }

                State::CommentEnd => {
                    match c {
                        '/' => self.comments_nested -= 1,
                        _ => self.state = State::CommentEat,
                    }

                    match self.comments_nested {
                        0 => self.state = State::Initial,
                        _ => self.state = State::CommentEat,
                    }
                    self.position += 1
                }

                State::CommentEat => {
                    match c {
                        '*' => self.state = State::CommentEnd,
                        '/' => self.state = State::CommentNested,
                        _ => {}
                    };
                    self.position += 1
                }

                State::And => {
                    return match c {
                        '&' => Ok(self.create_token(TokenKind::MulOP(MulOP::And), 2)),
                        _ => Err(self.create_error(ScannerErrorKind::IllegalCharacter(c), 0, None)),
                    }
                }

                State::Or => {
                    return match c {
                        '|' => {
                            Ok(self.create_token(TokenKind::AddOP(AddOP::Or), self.buffer.len()))
                        }
                        _ => Err(self.create_error(ScannerErrorKind::IllegalCharacter(c), 0, None)),
                    }
                }

                State::Relationship => {
                    return match c {
                        '=' => match self.buffer.as_str() {
                            "<" => Ok(self.create_token(TokenKind::RelOP(RelOP::LessEqual), 2)),
                            ">" => Ok(self.create_token(TokenKind::RelOP(RelOP::GreaterEqual), 2)),
                            "!" => Ok(self.create_token(TokenKind::RelOP(RelOP::NotEquals), 2)),
                            "=" => Ok(self.create_token(TokenKind::RelOP(RelOP::EqualsEquals), 2)),
                            _ => Err(self.create_error(
                                ScannerErrorKind::IllegalCharacter('a'),
                                1,
                                None,
                            )),
                        },
                        _ => match self.buffer.as_str() {
                            "<" => Ok(self.create_token(TokenKind::RelOP(RelOP::LessThan), 1)),
                            ">" => Ok(self.create_token(TokenKind::RelOP(RelOP::GreaterThan), 1)),
                            "!" => Ok(self.create_token(TokenKind::Delimiter(Delimiter::Not), 1)),
                            "=" => Ok(self.create_token(TokenKind::AssignOP, 1)),
                            _ => Err(self.create_error(
                                ScannerErrorKind::IllegalCharacter('a'),
                                1,
                                None,
                            )),
                        },
                    }
                }

                State::String => match c {
                    '"' => {
                        return Ok(self.create_token(
                            TokenKind::String(self.buffer[1..].to_string()),
                            self.buffer.len() - 1,
                        ))
                    }
                    '\n' => {
                        return Err(self.create_error(
                            ScannerErrorKind::InvalidStringLiteral,
                            self.buffer.len() - 1,
                            Some("expected '\"'".to_string()),
                        ))
                    }
                    _ => self.push_char(c),
                },
                State::CharLiteral => match c {
                    '\'' => {
                        return match self.buffer.len() {
                            0 | 1 => Err(self.create_error(
                                ScannerErrorKind::InvalidCharLiteral,
                                0,
                                None,
                            )),
                            2 => Ok(self.create_token(
                                TokenKind::CharLiteral(self.buffer.chars().nth(1).unwrap()),
                                1,
                            )),
                            len => Err(self.create_error(
                                ScannerErrorKind::InvalidCharLiteral,
                                len,
                                None,
                            )),
                        }
                    }
                    '\n' => {
                        self.push_char(c);
                        if self.buffer.len() > 2 {
                            return Err(self.create_error(
                                ScannerErrorKind::InvalidCharLiteral,
                                self.buffer.len(),
                                None,
                            ));
                        }
                    }
                    _ => self.push_char(c),
                },
            }
        }
        // When we run out of data in our source stream we return the EOF token
        Ok(self.create_token(TokenKind::Eof, 0))
    }

    pub fn create_token(&mut self, kind: TokenKind, len: usize) -> Token {
        let token = Token::new(kind, len, (self.lines_read, self.position));
        if self.debug.is_some() {
            println!("[SCANNER] {token}")
        }
        self.previous_location = (self.lines_read, self.position + 1);
        self.state = State::Initial;
        self.position += 1;
        token
    }
    fn create_error(
        &mut self,
        kind: ScannerErrorKind,
        len: usize,
        help: Option<String>,
    ) -> ScannerError {
        let location = (self.previous_location.0, self.previous_location.1);
        let line = match kind {
            ScannerErrorKind::MalformedNumber(_) => {
                let _ = self.stream.rewind();
                Some(self.stream.nth(location.0 - 1).unwrap())
            }
            _ => None,
        };
        ScannerError::new(
            kind,
            line,
            location,
            len,
            self.stream_name.to_string(),
            help,
        )
    }
    fn keyword_or_id_token(&mut self) -> Token {
        let kind = match self.buffer.as_str() {
            "break" => TokenKind::Keyword(Keyword::Break),
            "char" => TokenKind::Keyword(Keyword::Char),
            "case" => TokenKind::Keyword(Keyword::Case),
            "continue" => TokenKind::Keyword(Keyword::Continue),
            "default" => TokenKind::Keyword(Keyword::Default),
            "do" => TokenKind::Keyword(Keyword::Do),
            "else" => TokenKind::Keyword(Keyword::Else),
            "for" => TokenKind::Keyword(Keyword::For),
            "int" => TokenKind::Keyword(Keyword::Int),
            "if" => TokenKind::Keyword(Keyword::If),
            "newline" => TokenKind::Keyword(Keyword::Newline),
            "return" => TokenKind::Keyword(Keyword::Return),
            "read" => TokenKind::Keyword(Keyword::Read),
            "switch" => TokenKind::Keyword(Keyword::Switch),
            "while" => TokenKind::Keyword(Keyword::While),
            "write" => TokenKind::Keyword(Keyword::Write),
            data => TokenKind::Identifier(data.to_owned()),
        };
        self.create_token(kind, self.buffer.len())
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;
    use crate::scanner::token::{MulOP, RelOP};
    use crate::{
        scanner::token::{AddOP, Keyword, Token, TokenKind},
        BufferedStream,
    };
    use std::io::Cursor;

    #[test]
    fn test_scanner() {
        let data = "3+3";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(data.as_bytes())),
            "name.tc",
            None,
        );

        let mut t = scanner.next_token();
        assert_eq!(t, Ok(Token::new(TokenKind::Number(3.0), 1, (1, 0))));

        t = scanner.next_token();
        assert_eq!(t.unwrap().kind, TokenKind::AddOP(AddOP::Plus));
        //
        t = scanner.next_token();
        assert_eq!(t.unwrap().kind, TokenKind::Number(3.0));
    }

    #[test]
    fn sample_run() {
        const SAMPLE_DATA: &str = r#"hello char int while <= !=
                                123 "hello" /*
                                * / && /* ||
                                > */ */ *
                                // this is a comment"#;

        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        let mut tokens = vec![];
        loop {
            let token = scanner.next_token();
            match token {
                Ok(token) => {
                    if token.kind == TokenKind::Eof {
                        tokens.push(token.kind);
                        break;
                    }
                    tokens.push(token.kind);
                }
                Err(e) => {
                    println!("{e}");
                    break;
                }
            }
        }
        assert_eq!(
            tokens,
            [
                TokenKind::Identifier("hello".to_string()),
                TokenKind::Keyword(Keyword::Char),
                TokenKind::Keyword(Keyword::Int),
                TokenKind::Keyword(Keyword::While),
                TokenKind::RelOP(RelOP::LessEqual),
                TokenKind::RelOP(RelOP::NotEquals),
                TokenKind::Number(123.into()),
                TokenKind::String("hello".to_string()),
                TokenKind::MulOP(MulOP::Multiply),
                TokenKind::Eof
            ]
        )
    }

    #[test]
    fn pass_string_literal() {
        const SAMPLE_DATA: &str = r#""Hello world! :D""#;
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        assert_eq!(
            scanner.next_token().unwrap().kind,
            TokenKind::String("Hello world! :D".to_string())
        )
    }

    #[test]
    fn pass_complex_string_literal() {
        const SAMPLE_DATA: &str = "\"Hello \t\rd + a b c world! :D\"";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        assert_eq!(
            scanner.next_token().unwrap().kind,
            TokenKind::String("Hello \t\rd + a b c world! :D".to_string())
        )
    }

    #[test]
    fn fail_string_literal() {
        const SAMPLE_DATA: &str = "\"Hello world!\n\"";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        assert!(scanner.next_token().is_err())
    }

    #[test]
    fn pass_char_literal() {
        const SAMPLE_DATA: &str = "'\n'";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        assert_eq!(
            scanner.next_token().unwrap().kind,
            TokenKind::CharLiteral('\n')
        )
    }

    #[test]
    fn fail_char_literal_overrun() {
        const SAMPLE_DATA: &str = "'Hello world!";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        assert!(scanner.next_token().is_err())
    }
    #[test]
    fn fail_char_literal_unmatched() {
        const SAMPLE_DATA: &str = "'a!";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        assert!(scanner.next_token().is_err())
    }

    #[test]
    fn test_everything(){
        const SAMPLE_DATA : &str = r#"int char return if else for
                                    do while switch case default write
                                    read continue break newline
                                    a = 32; b = 32.; c7   =   99E+31"#;
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA)),
            "sample.tc",
            None,
        );
        let mut tokens = vec![];
        loop {
            let token = scanner.next_token();
            match token {
                Ok(token) => {
                    if token.kind == TokenKind::Eof {
                        tokens.push(token.kind);
                        break;
                    }
                    tokens.push(token.kind);
                }
                Err(e) => {
                    println!("{e}");
                    break;
                }
            }
        }
        println!("{:?}",tokens);
    }
}
