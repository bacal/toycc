use std::fmt::{Display, Formatter};

#[derive(Debug,Clone)]
pub enum RelOP{
    EqualsEquals,
    NotEquals,
    LessThan,
    LessEqual,
    GreaterEqual,
    GreaterThan,
}

#[derive(Debug,Clone)]
pub enum AddOP{
    Plus,
    Minus,
    Or,
}
#[derive(Debug,Clone)]
pub enum MulOP{
    Multiply,
    Divide,
    Mod,
    And,
}

#[derive(Debug,Clone)]
pub enum Token{
    // Keywords
    Break,
    Char, Case, Continue,
    Default, Do,
    Else,
    For,
    Int, If,
    Newline,
    Return, Read,
    Switch,
    While, Write,

    Identifier(String),
    Number(f64),
    CharLiteral(char),
    String(String),
    RelOP(RelOP),
    MulOP(MulOP),
    AddOP(AddOP),
    AssignOP,

    LParen, RParen, LCurly, RCurly, LBracket, RBracket,
    Comma, Semicolon, Not, Colon,

    Eof,
}

impl Display for RelOP{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self{
            RelOP::EqualsEquals => "==",
            RelOP::NotEquals => "!=",
            RelOP::LessThan => "<",
            RelOP::LessEqual => "<=",
            RelOP::GreaterEqual => ">=",
            RelOP::GreaterThan => ">",
        })
    }
}

impl Display for AddOP{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self{
            AddOP::Plus => "+",
            AddOP::Minus => "-",
            AddOP::Or => "||",
        })
    }
}

impl Display for MulOP{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self{
            MulOP::Multiply => "*",
            MulOP::Divide => "/",
            MulOP::Mod => "%",
            MulOP::And => "&&",
        })
    }
}

impl Display for Token{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let opstr;
        let pair = match self{
            Token::Int => ("INT","int"),
            Token::Char => ("CHAR","char"),
            Token::Return => ("RETURN","return"),
            Token::If => ("IF","if"),
            Token::Else => ("ELSE","else"),
            Token::For => ("FOR","for"),
            Token::Do => ("DO","do"),
            Token::While => ("WHILE","while"),
            Token::Switch => ("SWITCH","switch"),
            Token::Case => ("CASE","case"),
            Token::Default => ("DEFAULT","default"),
            Token::Write => ("WRITE","write"),
            Token::Read => ("READ","read"),
            Token::Continue => ("CONTINUE","continue"),
            Token::Break => ("BREAK","break"),
            Token::Newline => ("NEWLINE","newline"),
            Token::Identifier(id) => ("ID",id.as_str()),
            Token::Number(num) => {
                opstr = num.to_string();
                ("NUMBER",opstr.as_str())
            },
            Token::CharLiteral(c) => {
                opstr = format!("'{}'",c);
                ("CHAR_LITERAL",opstr.as_str())
            },
            Token::String(string) => {
                opstr = format!("\"{}\"",string);
                ("STRING",opstr.as_str())
            },
            Token::RelOP(op) => {
                opstr = op.to_string();
                ("RELOP",opstr.as_str())
            },
            Token::MulOP(op) => {
                opstr = op.to_string();
                ("MULOP",opstr.as_str())
            },
            Token::AddOP(op) => {
                opstr = op.to_string();
                ("ADDOP",opstr.as_str())
            },
            Token::AssignOP => ("ASSIGNOP","="),
            Token::LParen => ("LPAREN","("),
            Token::RParen => ("RPAREN",")"),
            Token::LCurly => ("LCURLY","{"),
            Token::RCurly => ("RCURLY","}"),
            Token::LBracket => ("LBRACKET","["),
            Token::RBracket => ("RBRACKET","]"),
            Token::Comma => ("COMMA",","),
            Token::Semicolon => ("SEMICOLON",";"),
            Token::Not => ("NOT","!"),
            Token::Colon => ("COLON",":"),
            Token::Eof => ("EOF", "EOF"),
        };
        write!(f,"(<{}>,\"{}\")",pair.0,pair.1)
    }
}