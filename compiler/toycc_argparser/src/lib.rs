mod error;

use std::env::{args};
use colored::Colorize;
use itertools::Itertools;
use crate::error::ArgumentParseError;

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const USAGE: &str = r"toycc [options] [input]...";
const OPTIONS:  &str = r#"
  -debug <level>  Display messages that aid in tracing the compilation process.
                       0 - all messages
                       1 - scanner messages only
  -verbose        Display all information
  -help           Print help"#;

#[derive(Debug, Eq, PartialEq)]
pub struct Arguments{
    pub help: bool,
    pub authors: bool,
    pub debug: Option<u32>,
    pub verbose: bool,
    pub file_names: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
enum Argument{
    Help,
    Authors,
    Debug,
    Verbose,
    Positional(String),
}

#[derive(Debug, Eq, PartialEq)]
enum Token{
    Argument(Argument),
    Number(u32),
    Eos,
}

enum ScannerState{
    Initial,
    Argument,
    Number,
    Positional,
}

impl Arguments{
    pub fn print_usage(){
        println!("{}: {USAGE}\n\n{DESCRIPTION}\n\n{}\n\n{}: {OPTIONS}",
                 "Usage".white().bold(),
                 Self::authors_string(),
                 "Options".white().bold());
    }
    fn authors_string() -> String{
        let authors = AUTHORS.split(":")
            .filter(|s| !s.contains("<"))
            .join(", ");
        format!("{}: {authors}","Authors".white().bold())
    }
    pub fn print_authors(){
        println!("{}: {}","toycc".white().bold(),Self::authors_string())
    }

    pub fn parse() -> Result<Self, ArgumentParseError>{

        let input = args().skip(1).join(" ");
        let mut args = Arguments{
            help: false,
            authors: false,
            debug: None,
            verbose: false,
            file_names: vec![],
        };
        let mut tokens = scan_tokens(&input)?.into_iter();
        while let Some(token) = tokens.next(){
            match token{
                Token::Argument(Argument::Debug) =>{
                    match tokens.next(){
                        Some(Token::Number(num)) =>{
                            match num{
                                0 | 1 => args.debug = Some(num),
                                _ => return Err(ArgumentParseError::InvalidDebug(num)),
                            }
                        }
                        _ => return Err(ArgumentParseError::MissingValue),
                    }
                }
                Token::Argument(Argument::Authors) => args.authors = true,
                Token::Argument(Argument::Verbose) => args.verbose = true,
                Token::Argument(Argument::Help) => args.help = true,
                Token::Argument(Argument::Positional(s)) => args.file_names.push(s.clone()),
                Token::Eos =>{},
                Token::Number(n) =>  args.file_names.push(n.to_string()),
            }
        }
        Ok(args)
    }
}

fn scan_tokens(input: &str) -> Result<Vec<Token>,ArgumentParseError>{
    let input = input.to_string() + " ";
    let mut tokens = vec![];
    let mut state = ScannerState::Initial;
    let mut buffer = String::new();
    let mut input = input.chars().peekable();
    while let Some(c) = input.next(){
        match state{
            ScannerState::Initial => {
                buffer.clear();
                match c{
                    '-' => state= ScannerState::Argument,
                    ('0'..='9') => {
                        state = ScannerState::Number;
                        buffer.push(c);
                    },
                    _ => {
                        state = ScannerState::Positional;
                        buffer.push(c);
                    },
                }
            }
            ScannerState::Number => {
                match c{
                    ' ' | '\t' | '\n' => {
                        tokens.push(Token::Number(buffer.parse::<u32>().unwrap()));
                        state = ScannerState::Initial;
                    },
                    _ => state = ScannerState::Positional,
                }
                buffer.push(c);
            }
            ScannerState::Positional =>{
                match c{
                    ' ' | '\t' | '\n' => {
                        tokens.push(Token::Argument(Argument::Positional(buffer.clone())));
                        state = ScannerState::Initial;
                    },
                    _ => buffer.push(c)
                }
            }
            ScannerState::Argument =>{
                match c{
                    ' ' => {
                        state = ScannerState::Initial;
                        tokens.push(Token::Argument(buffer.as_str().try_into()?));
                    },
                    _ => {
                        buffer.push(c);
                    },
                }
            }

        }
    }
    tokens.push(Token::Eos);
    Ok(tokens)
}

#[cfg(test)]
mod scanner_tests{
    use super::{scan_tokens, Argument, Token};

    #[test]
    fn test_help(){
       assert_eq!(scan_tokens("-help 2"), Ok(vec![Token::Argument(Argument::Help),Token::Number(2),Token::Eos]))
    }

    #[test]
    fn test_debug(){
        assert_eq!(scan_tokens("2 -debug 2"),
                   Ok(vec![Token::Number(2), Token::Argument(Argument::Debug),Token::Number(2),Token::Eos]))
    }

    #[test]
    fn test_positional(){
        assert_eq!(scan_tokens("2a.c -debug 2"),
                   Ok(vec![Token::Argument(Argument::Positional("2a.c".to_string())),
                           Token::Argument(Argument::Debug),Token::Number(2),Token::Eos]))
    }

    #[test]
    fn test_positional2(){
        assert_eq!(scan_tokens("2a.c -debug a223.c"),
                   Ok(vec![Token::Argument(Argument::Positional("2a.c".to_string())),
                           Token::Argument(Argument::Debug),
                           Token::Argument(Argument::Positional("a223.c".to_string())),
                           Token::Eos]))
    }
}

impl TryFrom<&str> for Argument{
    type Error = ArgumentParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value{
            "debug" => Ok(Argument::Debug),
            "verbose" => Ok(Argument::Verbose),
            "help" => Ok(Argument::Help),
            "authors" => Ok(Argument::Authors),
            _ => Err(ArgumentParseError::UnknownArgument(value.to_string())),
        }
    }
}