use crate::scanner::error::{ScannerError, ScannerErrorKind};
use crate::scanner::token::{Delimiter, Keyword, RelOP, Token, TokenKind, Type};
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
    SciSign,
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

pub struct Scanner<S: Read + Seek> {
    pub stream: BufferedStream<S>,
    buffer: String,
    state: State,
    position: usize,
    debug: Option<u32>,
    lines_read: usize,
    comments_nested: Vec<(usize, usize)>,
    pub(crate) previous_location: (usize, usize),
}

impl<S: Read + Seek> Scanner<S> {
    pub fn new(stream: BufferedStream<S>, debug: Option<u32>) -> Self {
        Self {
            debug,
            stream,
            state: State::Initial,
            buffer: String::new(),
            lines_read: 0,
            comments_nested: vec![],
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
                        ';' => {
                            return Ok(
                                self.create_token(TokenKind::Delimiter(Delimiter::Semicolon), 1)
                            )
                        }
                        '(' => {
                            return Ok(self.create_token(TokenKind::Delimiter(Delimiter::LParen), 1))
                        }
                        ')' => {
                            return Ok(self.create_token(TokenKind::Delimiter(Delimiter::RParen), 1))
                        }
                        '[' => {
                            return Ok(
                                self.create_token(TokenKind::Delimiter(Delimiter::LBracket), 1)
                            )
                        }
                        ']' => {
                            return Ok(
                                self.create_token(TokenKind::Delimiter(Delimiter::RBracket), 1)
                            )
                        }
                        '{' => {
                            return Ok(self.create_token(TokenKind::Delimiter(Delimiter::LCurly), 1))
                        }
                        '}' => {
                            return Ok(self.create_token(TokenKind::Delimiter(Delimiter::RCurly), 1))
                        }
                        ':' => {
                            return Ok(self.create_token(TokenKind::Delimiter(Delimiter::Colon), 1))
                        }
                        ',' => {
                            return Ok(self.create_token(TokenKind::Delimiter(Delimiter::Comma), 1))
                        }
                        _ => {
                            println!(
                                "{}",
                                self.create_error(ScannerErrorKind::IllegalCharacter(c), 0, None)
                            );
                            self.change_state(State::Initial, c);
                        }
                    }
                }

                State::Identifier => match c {
                    ('a'..='z') | ('A'..='Z') | ('0'..='9') => self.push_char(c),
                    _ => {
                        if !" \t\n".contains(c) {
                            self.position -= 1;
                        }
                        return Ok(self.keyword_or_id_token());
                    }
                },

                State::Integer => match c {
                    ('0'..='9') => self.push_char(c),
                    'E' => self.change_state(State::SciSign, c),
                    '.' => self.change_state(State::FloatFirst, c),
                    _ => {
                        return match self.buffer.parse::<f64>() {
                            Ok(num) => {
                                if !" \t\n".contains(c) {
                                    self.position -= 1;
                                }
                                Ok(self.create_token(
                                    TokenKind::Number { num, sci: false },
                                    self.buffer.len(),
                                ))
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
                State::SciSign => match c {
                    '+' | '-' => self.change_state(State::SciFirst, c),
                    ('0'..='9') => self.change_state(State::Scientific, c),
                    _ => {
                        return Err(self.create_error(
                            ScannerErrorKind::MalformedNumber(
                                "scientific number missing sign or digit".to_string(),
                            ),
                            1,
                            Some("expected +, -, or digit".to_string()),
                        ))
                    }
                },

                State::FloatFirst => match c {
                    ('0'..='9') => self.change_state(State::Float, c),
                    _ => {
                        return Err(self.create_error(
                            ScannerErrorKind::MalformedNumber(
                                "missing digits after decimal".to_string(),
                            ),
                            self.buffer.len(),
                            Some("expected digit".to_string()),
                        ))
                    }
                },

                State::SciFirst => match c {
                    ('0'..='9') => self.change_state(State::Scientific, c),
                    _ => {
                        return Err(self.create_error(
                            ScannerErrorKind::MalformedNumber(
                                "missing digits after sign".to_string(),
                            ),
                            self.buffer.len(),
                            Some("expected digit".to_string()),
                        ))
                    }
                },
                State::Float => match c {
                    ('0'..='9') => self.push_char(c),
                    'E' => self.change_state(State::SciSign, c),
                    _ => {
                        if !" \t\n".contains(c) {
                            self.position -= 1;
                        }
                        return Ok(self.create_token(
                            TokenKind::Number {
                                num: self.buffer.parse::<f64>().unwrap(),
                                sci: false,
                            },
                            self.buffer.len(),
                        ));
                    }
                },

                State::Scientific => match c {
                    ('0'..='9') => self.push_char(c),
                    'E' => {
                        return Err(self.create_error(
                            ScannerErrorKind::MalformedNumber("duplicate E is invalid".to_string()),
                            1,
                            None,
                        ))
                    }
                    _ => {
                        if !" \t\n".contains(c) {
                            self.position -= 1;
                        }
                        return Ok(self.create_token(
                            TokenKind::Number {
                                num: self.buffer.parse::<f64>().unwrap(),
                                sci: true,
                            },
                            self.buffer.len(),
                        ));
                    }
                },

                State::CommentStart => match c {
                    '/' => {
                        self.next_line();
                        self.state = State::Initial;
                    }
                    '*' => {
                        self.change_state(State::CommentEat, c);
                        self.comments_nested.push((self.lines_read, self.position));
                    }
                    _ => {
                        if !" \t\n".contains(c) {
                            self.position -= 1;
                        }
                        return Ok(self.create_token(TokenKind::MulOP(MulOP::Divide), 1));
                    }
                },

                State::CommentNested => {
                    if c == '*' {
                        self.comments_nested.push((self.lines_read, self.position));
                    }
                    self.state = State::CommentEat;
                }

                State::CommentEnd => {
                    match c {
                        '/' => {
                            self.comments_nested.pop();
                        }
                        _ => self.state = State::CommentEat,
                    }

                    match self.comments_nested.len() {
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
                        _ => Err(self.create_error(ScannerErrorKind::InvalidMulOp, 1, None)),
                    }
                }

                State::Or => {
                    return match c {
                        '|' => {
                            Ok(self.create_token(TokenKind::AddOP(AddOP::Or), self.buffer.len()))
                        }
                        _ => Err(self.create_error(ScannerErrorKind::InvalidAddOp, 1, None)),
                    }
                }

                State::Relationship => {
                    return match c {
                        '=' => match self.buffer.as_str() {
                            "<" => Ok(self.create_token(TokenKind::RelOP(RelOP::LessEqual), 2)),
                            ">" => Ok(self.create_token(TokenKind::RelOP(RelOP::GreaterEqual), 2)),
                            "!" => Ok(self.create_token(TokenKind::RelOP(RelOP::NotEquals), 2)),
                            "=" => Ok(self.create_token(TokenKind::RelOP(RelOP::EqualsEquals), 2)),
                            _ => Err(self.create_error(ScannerErrorKind::InvalidRelOp, 1, None)),
                        },
                        _ => {
                            if !" \t\n".contains(c) {
                                self.position -= 1;
                            }
                            match self.buffer.as_str() {
                                "<" => Ok(self.create_token(TokenKind::RelOP(RelOP::LessThan), 1)),
                                ">" => {
                                    Ok(self.create_token(TokenKind::RelOP(RelOP::GreaterThan), 1))
                                }
                                "!" => {
                                    Ok(self.create_token(TokenKind::Delimiter(Delimiter::Not), 1))
                                }
                                "=" => Ok(self.create_token(TokenKind::AssignOP, 1)),
                                _ => {
                                    Err(self.create_error(ScannerErrorKind::InvalidRelOp, 1, None))
                                }
                            }
                        }
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
                            (0..=2) => Ok(self.create_token(
                                TokenKind::CharLiteral(self.buffer.chars().nth(1)),
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
        match self.comments_nested.len() {
            0 => Ok(self.create_token(TokenKind::Eof, 0)),
            _ => Err(self.create_error(ScannerErrorKind::UnterminatedComment, 1, None)),
        }
    }

    fn create_token(&mut self, kind: TokenKind, len: usize) -> Token {
        let token = Token::new(kind, len, self.previous_location);
        if self.debug.is_some() {
            println!("[SCANNER] token {token}")
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
        let location = match kind {
            ScannerErrorKind::UnterminatedComment => self.comments_nested.pop().unwrap(),
            _ => {
                if (self.lines_read, self.position) != self.previous_location {
                    (self.previous_location.0, self.previous_location.1 + 1)
                } else {
                    (self.lines_read, self.position + 1)
                }
            }
        };
        let line = match kind {
            ScannerErrorKind::IllegalCharacter(_) => None,
            _ => self.error_get_line(location),
        };
        ScannerError::new(
            kind,
            line,
            location,
            len,
            self.stream.name.clone().unwrap_or_default(),
            help,
        )
    }

    pub(crate) fn error_get_line(&mut self, location: (usize, usize)) -> Option<String> {
        let _ = self.stream.rewind();
        Some(
            self.stream
                .nth(location.0 - 1)
                .unwrap_or_default()
                .trim_end()
                .to_string(),
        )
    }

    fn keyword_or_id_token(&mut self) -> Token {
        let kind = match self.buffer.as_str() {
            "char" => TokenKind::Type(Type::Char),
            "int" => TokenKind::Type(Type::Int),
            "break" => TokenKind::Keyword(Keyword::Break),
            "case" => TokenKind::Keyword(Keyword::Case),
            "continue" => TokenKind::Keyword(Keyword::Continue),
            "default" => TokenKind::Keyword(Keyword::Default),
            "do" => TokenKind::Keyword(Keyword::Do),
            "else" => TokenKind::Keyword(Keyword::Else),
            "for" => TokenKind::Keyword(Keyword::For),
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
    use crate::scanner::token::{MulOP, RelOP, Type};
    use crate::{
        scanner::token::{AddOP, Keyword, Token, TokenKind},
        BufferedStream,
    };
    use std::io::Cursor;

    #[test]
    fn test_scanner() {
        let data = "3+3";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(data.as_bytes()), Some("sample.tc".to_string())),
            None,
        );

        let mut t = scanner.next_token();
        assert_eq!(
            t,
            Ok(Token::new(
                TokenKind::Number {
                    num: 3.0,
                    sci: false
                },
                1,
                (1, 1),
            ))
        );

        t = scanner.next_token();
        assert_eq!(t.unwrap().kind, TokenKind::AddOP(AddOP::Plus));
        //
        t = scanner.next_token();
        assert_eq!(
            t.unwrap().kind,
            TokenKind::Number {
                num: 3.0,
                sci: false
            }
        );
    }

    #[test]
    fn sample_run() {
        const SAMPLE_DATA: &str = r#"hello char int while <= !=
                                123 "hello" /*
                                * / && /* ||
                                > */ */ *
                                // this is a comment"#;

        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
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
                TokenKind::Type(Type::Char),
                TokenKind::Type(Type::Int),
                TokenKind::Keyword(Keyword::While),
                TokenKind::RelOP(RelOP::LessEqual),
                TokenKind::RelOP(RelOP::NotEquals),
                TokenKind::Number {
                    num: 123.0,
                    sci: false
                },
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
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
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
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
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
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
            None,
        );
        assert!(scanner.next_token().is_err())
    }

    #[test]
    fn pass_char_literal() {
        const SAMPLE_DATA: &str = "'\n'";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
            None,
        );
        assert_eq!(
            scanner.next_token().unwrap().kind,
            TokenKind::CharLiteral(Some('\n'))
        )
    }

    #[test]
    fn fail_char_literal_overrun() {
        const SAMPLE_DATA: &str = "'Hello world!";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
            None,
        );
        assert!(scanner.next_token().is_err())
    }
    #[test]
    fn fail_char_literal_unmatched() {
        const SAMPLE_DATA: &str = "'a!";
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
            None,
        );
        assert!(scanner.next_token().is_err())
    }

    #[test]
    fn test_everything() {
        const SAMPLE_DATA: &str = r#"int char return if else for
                                    do while switch case default write
                                    read continue break newline
                                    a = 32; b = 32.0; c7   =   99E+31
                                    // This is a comment
                                    /* multiline
                                    comment */
                                    f = 2.0E+1E3"#;
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
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
        println!("{:?}", tokens);
    }

    #[test]
    fn test_number_invalid_exp() {
        const SAMPLE_DATA: &str = r#"2E+1E1"#;
        let mut scanner = Scanner::new(
            BufferedStream::new(Cursor::new(SAMPLE_DATA), Some("sample.tc".to_string())),
            None,
        );
        let mut t = scanner.next_token();
        t = scanner.next_token();
        assert!(t.is_err())
    }
}
