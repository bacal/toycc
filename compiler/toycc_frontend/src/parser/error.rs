use crate::scanner::error::ScannerError;
use crate::scanner::token::{Delimiter, Keyword};
use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};

#[derive(Debug)]
pub enum ParserErrorKind {
    ScannerError(ScannerError),
    Generic,
    ExpectedType,
    ExpectedIdentifier,
    ExpectedDelimiter(Delimiter),
    ExpectedKeyword(Keyword),
    ExpectedNumber,
}
impl Default for ParserErrorKind {
    fn default() -> Self {
        Self::Generic
    }
}

#[derive(Debug, Default, Report)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    line: Option<String>,
    location: (usize, usize),
    len: usize,
    stream_name: String,
    help: Option<String>,
}

impl From<ScannerError> for ParserError {
    fn from(scanner_error: ScannerError) -> Self {
        Self {
            kind: ParserErrorKind::ScannerError(scanner_error),
            ..Default::default()
        }
    }
}
impl From<ScannerError> for Box<ParserError> {
    fn from(value: ScannerError) -> Self {
        Box::new(ParserError::from(value))
    }
}
impl From<Box<ScannerError>> for Box<ParserError> {
    fn from(value: Box<ScannerError>) -> Self {
        Box::new(ParserError::from(*value))
    }
}

impl ParserError {
    pub fn new(
        kind: ParserErrorKind,
        line: Option<String>,
        location: (usize, usize),
        len: usize,
        stream_name: String,
        help: Option<String>,
    ) -> Self {
        Self {
            kind,
            line,
            location,
            len,
            stream_name,
            help,
        }
    }
}

impl Diagnostic for ParserError {
    fn info(&self) -> String {
        match &self.kind {
            ParserErrorKind::ScannerError(s) => s.info(),
            ParserErrorKind::ExpectedDelimiter(d) => format!("expected delimiter: '{d}'"),
            ParserErrorKind::Generic => String::new(),
            ParserErrorKind::ExpectedType => "expected type".to_string(),
            ParserErrorKind::ExpectedIdentifier => "expected identifier".to_string(),
            ParserErrorKind::ExpectedKeyword(keyword) => format!("expected {keyword}"),
            ParserErrorKind::ExpectedNumber => "expected number".to_string(),
        }
    }

    fn level(&self) -> ReportLevel {
        match &self.kind {
            ParserErrorKind::ScannerError(s) => s.level(),
            _ => ReportLevel::Error(ErrorKind::ParsingError {
                file_name: self.stream_name.clone(),
                pos: self.location,
                len: self.len,
                source: self.line.clone(),
            }),
        }
    }

    fn help(&self) -> Option<String> {
        match &self.kind {
            ParserErrorKind::ScannerError(s) => s.help().to_owned(),
            _ => self.help.to_owned(),
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        match &self.kind {
            ParserErrorKind::ScannerError(s) => s.others(),
            _ => None,
        }
    }
}
