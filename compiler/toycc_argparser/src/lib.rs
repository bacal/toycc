mod error;

use crate::error::ArgumentParseError;
use colored::Colorize;
use itertools::Itertools;
use std::env::args;

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const USAGE: &str = r"toycc [options] input_file";
const OPTIONS: &str = r#"
    -help               display a usage message
    -output <file>      specifies target file name
    -class  <file>      specifies class file name
    -debug  <level>     display messages that aid in tracing the
                        compilation process. If level is:
                            0 - all messages
                            1 - scanner messages only
                            2 - parser messages only
                            3 - code generation messages only
    -abstract           dump the abstract syntax tree
    -symbol             dump the symbol table(s)
    -code               dump the generated program
    -verbose            display all information
    -version            display the program version"#;

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Arguments {
    pub help: bool,
    pub authors: bool,
    pub debug: Option<u32>,
    pub class: Option<String>,
    pub output: Option<String>,
    pub dump_ast: bool,
    pub dump_sym: bool,
    pub dump_cgn: bool,
    pub version: bool,
    pub verbose: bool,
    pub file_name: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
enum Argument {
    Help,
    Authors,
    Debug,
    Verbose,
    Positional(String),
    DumpAST,
    DumpSYM,
    DumpCGN,
    Version,
    Class,
    Output,
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    Argument(Argument),
    Number(u32),
    Eos,
}

enum ScannerState {
    Initial,
    Argument,
    Number,
    Positional,
}

impl Arguments {
    pub fn print_usage() {
        println!(
            "{}: {USAGE}\n\n{}: {OPTIONS}\n{}",
            "Usage".white().bold(),
            "Options".white().bold(),
            Self::contacts_string(),
        );
    }
    fn authors_string() -> String {
        let authors = AUTHORS.split(':').filter(|s| !s.contains('<')).join(", ");
        format!("{}: {authors}", "Authors".white().bold())
    }

    fn contacts_string() -> String {
        let authors = AUTHORS.split(':').filter(|s| s.contains('<')).join(", ");
        format!("Send bug reports to: {authors}")
    }

    pub fn print_authors() {
        println!("{}: {}", "toycc".white().bold(), Self::authors_string())
    }

    pub fn parse() -> Result<Self, ArgumentParseError> {
        let input = args().skip(1).join(" ");
        let mut args = Arguments::default();
        let mut tokens = scan_tokens(&input)?.into_iter();
        while let Some(token) = tokens.next() {
            match token {
                Token::Argument(Argument::Debug) => match tokens.next() {
                    Some(Token::Number(num)) => match num {
                        (0..=3) => args.debug = Some(num),
                        _ => return Err(ArgumentParseError::InvalidDebug(num)),
                    },
                    _ => return Err(ArgumentParseError::MissingValue("debug")),
                },
                Token::Argument(Argument::Authors) => args.authors = true,
                Token::Argument(Argument::Verbose) => args.verbose = true,
                Token::Argument(Argument::Help) => args.help = true,
                Token::Number(s) => match args.file_name {
                    Some(_) => return Err(ArgumentParseError::UnknownArgument(s.to_string())),
                    None => args.file_name = Some(s.to_string()),
                },
                Token::Argument(Argument::Positional(s)) => match args.file_name {
                    Some(_) => return Err(ArgumentParseError::ExtraPositional(s.clone())),
                    None => args.file_name = Some(s.clone()),
                },
                Token::Argument(Argument::DumpAST) => args.dump_ast = true,
                Token::Argument(Argument::DumpSYM) => args.dump_sym = true,
                Token::Argument(Argument::DumpCGN) => args.dump_cgn = true,
                Token::Argument(Argument::Version) => args.version = true,
                Token::Argument(Argument::Class) => match tokens.next() {
                    Some(Token::Argument(Argument::Positional(s))) => args.class = Some(s.clone()),
                    _ => return Err(ArgumentParseError::MissingValue("class")),
                },
                Token::Argument(Argument::Output) => match tokens.next() {
                    Some(Token::Argument(Argument::Positional(s))) => args.output = Some(s.clone()),
                    _ => return Err(ArgumentParseError::MissingValue("output")),
                },
                Token::Eos => {}
            }
        }
        Ok(args)
    }
}

fn scan_tokens(input: &str) -> Result<Vec<Token>, ArgumentParseError> {
    let input = input.to_string() + " ";
    let mut tokens = vec![];
    let mut state = ScannerState::Initial;
    let mut buffer = String::new();
    let input = input.chars().peekable();
    for c in input {
        match state {
            ScannerState::Initial => {
                buffer.clear();
                match c {
                    '-' => state = ScannerState::Argument,
                    ('0'..='9') => {
                        state = ScannerState::Number;
                        buffer.push(c);
                    }
                    _ => {
                        state = ScannerState::Positional;
                        buffer.push(c);
                    }
                }
            }
            ScannerState::Number => {
                match c {
                    ' ' | '\t' | '\n' => {
                        tokens.push(Token::Number(buffer.parse::<u32>().unwrap()));
                        state = ScannerState::Initial;
                    }
                    ('0'..='9') => {}
                    _ => state = ScannerState::Positional,
                }
                buffer.push(c);
            }
            ScannerState::Positional => match c {
                ' ' | '\t' | '\n' => {
                    tokens.push(Token::Argument(Argument::Positional(buffer.clone())));
                    state = ScannerState::Initial;
                }
                _ => buffer.push(c),
            },
            ScannerState::Argument => match c {
                ' ' => {
                    state = ScannerState::Initial;
                    tokens.push(Token::Argument(buffer.as_str().try_into()?));
                }
                _ => {
                    buffer.push(c);
                }
            },
        }
    }
    tokens.push(Token::Eos);
    Ok(tokens)
}

impl TryFrom<&str> for Argument {
    type Error = ArgumentParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "debug" => Ok(Argument::Debug),
            "verbose" => Ok(Argument::Verbose),
            "help" => Ok(Argument::Help),
            "authors" => Ok(Argument::Authors),
            "abstract" => Ok(Argument::DumpAST),
            "symbol" => Ok(Argument::DumpSYM),
            "code" => Ok(Argument::DumpCGN),
            "version" => Ok(Argument::Version),
            "class" => Ok(Argument::Class),
            "output" => Ok(Argument::Output),
            _ => Err(ArgumentParseError::UnknownArgument(value.to_string())),
        }
    }
}

#[cfg(test)]
mod scanner_tests {
    use super::{scan_tokens, Argument, Token};

    #[test]
    fn test_help() {
        assert_eq!(
            scan_tokens("-help 2"),
            Ok(vec![
                Token::Argument(Argument::Help),
                Token::Number(2),
                Token::Eos
            ])
        )
    }

    #[test]
    fn test_debug() {
        assert_eq!(
            scan_tokens("2 -debug 2"),
            Ok(vec![
                Token::Number(2),
                Token::Argument(Argument::Debug),
                Token::Number(2),
                Token::Eos
            ])
        )
    }

    #[test]
    fn test_positional() {
        assert_eq!(
            scan_tokens("2a.c -debug 2"),
            Ok(vec![
                Token::Argument(Argument::Positional("2a.c".to_string())),
                Token::Argument(Argument::Debug),
                Token::Number(2),
                Token::Eos
            ])
        )
    }

    #[test]
    fn test_positional2() {
        assert_eq!(
            scan_tokens("2a.c -debug 9999 a223.c"),
            Ok(vec![
                Token::Argument(Argument::Positional("2a.c".to_string())),
                Token::Argument(Argument::Debug),
                Token::Number(9999),
                Token::Argument(Argument::Positional("a223.c".to_string())),
                Token::Eos
            ])
        )
    }
}
