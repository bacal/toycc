use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum RelOP {
    EqualsEquals,
    NotEquals,
    LessThan,
    LessEqual,
    GreaterEqual,
    GreaterThan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddOP {
    Plus,
    Minus,
    Or,
}
#[derive(Debug, Clone, PartialEq)]
pub enum MulOP {
    Multiply,
    Divide,
    Mod,
    And,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Break,
    Case,
    Continue,
    Default,
    Do,
    Else,
    For,
    If,
    Newline,
    Return,
    Read,
    Switch,
    While,
    Write,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Char,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Delimiter {
    LParen,
    RParen,
    LCurly,
    RCurly,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Not,
    Colon,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Type(Type),
    Identifier(String),
    Number { num: f64, sci: bool },
    CharLiteral(Option<char>),
    String(String),
    RelOP(RelOP),
    MulOP(MulOP),
    AddOP(AddOP),
    AssignOP,
    Delimiter(Delimiter),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub len: usize,
    pub location: (usize, usize),
}

impl Token {
    pub fn new(kind: TokenKind, len: usize, location: (usize, usize)) -> Self {
        Self {
            kind,
            len,
            location,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}
impl Display for RelOP {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RelOP::EqualsEquals => "==",
                RelOP::NotEquals => "!=",
                RelOP::LessThan => "<",
                RelOP::LessEqual => "<=",
                RelOP::GreaterEqual => ">=",
                RelOP::GreaterThan => ">",
            }
        )
    }
}

impl Display for AddOP {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AddOP::Plus => "+",
                AddOP::Minus => "-",
                AddOP::Or => "||",
            }
        )
    }
}

impl Display for MulOP {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MulOP::Multiply => "*",
                MulOP::Divide => "/",
                MulOP::Mod => "%",
                MulOP::And => "&&",
            }
        )
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Return => "RETURN",
                Self::If => "IF",
                Self::Else => "ELSE",
                Self::For => "FOR",
                Self::Do => "DO",
                Self::While => "WHILE",
                Self::Switch => "SWITCH",
                Self::Case => "CASE",
                Self::Default => "DEFAULT",
                Self::Write => "WRITE",
                Self::Read => "READ",
                Self::Continue => "CONTINUE",
                Self::Break => "BREAK",
                Self::Newline => "NEWLINE",
            }
        )
    }
}

impl Display for Delimiter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LParen => "(",
                Self::RParen => ")",
                Self::LCurly => "{",
                Self::RCurly => "}",
                Self::LBracket => "[",
                Self::RBracket => "]",
                Self::Comma => ",",
                Self::Semicolon => ";",
                Self::Not => "!",
                Self::Colon => ":",
            }
        )
    }
}

impl<'a> From<Delimiter> for (&'a str, &'a str) {
    fn from(value: Delimiter) -> Self {
        match value {
            Delimiter::LParen => ("LPAREN", "("),
            Delimiter::RParen => ("RPAREN", ")"),
            Delimiter::LCurly => ("LCURLY", "{"),
            Delimiter::RCurly => ("RCURLY", "}"),
            Delimiter::LBracket => ("LBRACKET", "["),
            Delimiter::RBracket => ("RBRACKET", "]"),
            Delimiter::Comma => ("COMMA", ","),
            Delimiter::Semicolon => ("SEMICOLON", ";"),
            Delimiter::Not => ("NOT", "!"),
            Delimiter::Colon => ("COLON", ":"),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Int => "INT",
                Type::Char => "CHAR",
            }
        )
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let _buf1;
        let _buf2;
        let pair = match self {
            Self::Keyword(keyword) => {
                _buf1 = keyword.to_string();
                _buf2 = _buf1.to_lowercase();
                (_buf1.as_str(), _buf2.as_str())
            }
            Self::Identifier(id) => ("ID", id.as_str()),
            Self::Number { num, sci } => {
                _buf1 = match sci {
                    true => format!("{:E}", num),
                    false => num.to_string(),
                };
                ("NUMBER", _buf1.as_str())
            }
            Self::CharLiteral(c) => {
                _buf1 = format!(
                    "'{}'",
                    match c {
                        Some(c) => c.escape_debug().to_string(),
                        None => "".to_string(),
                    }
                );
                ("CHAR_LITERAL", _buf1.as_str())
            }
            Self::String(string) => {
                _buf1 = format!("\"{}\"", string);
                ("STRING", _buf1.as_str())
            }
            Self::RelOP(op) => {
                _buf1 = op.to_string();
                ("RELOP", _buf1.as_str())
            }
            Self::MulOP(op) => {
                _buf1 = op.to_string();
                ("MULOP", _buf1.as_str())
            }
            Self::AddOP(op) => {
                _buf1 = op.to_string();
                ("ADDOP", _buf1.as_str())
            }
            Self::AssignOP => ("ASSIGNOP", "="),
            Self::Eof => ("EOF", "EOF"),
            Self::Delimiter(del) => del.clone().into(),
            TokenKind::Type(t) => {
                _buf1 = t.to_string();
                _buf2 = _buf1.to_lowercase();
                (_buf1.as_str(), _buf2.as_str())
            }
        };
        write!(f, "{} {}", pair.0, pair.1)
    }
}
