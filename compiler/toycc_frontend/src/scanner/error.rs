use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel, WarningKind};

#[derive(Debug, PartialEq)]
pub enum ScannerErrorKind {
    IllegalCharacter(char),
    MalformedNumber(String),
    InvalidCharLiteral,
    InvalidStringLiteral,
}

#[derive(Debug, Report, PartialEq)]
pub struct ScannerError {
    pub kind: ScannerErrorKind,
    line: Option<String>,
    location: (usize, usize),
    len: usize,
    stream_name: String,
    help: Option<String>,
}

impl ScannerError {
    pub fn new(
        kind: ScannerErrorKind,
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

impl Diagnostic for ScannerError {
    fn info(&self) -> String {
        match &self.kind {
            ScannerErrorKind::IllegalCharacter(c) => {
                format!("illegal character: '{}'", c.escape_debug())
            }
            ScannerErrorKind::MalformedNumber(c) => c.to_string(),
            ScannerErrorKind::InvalidCharLiteral => "invalid char literal".to_string(),
            ScannerErrorKind::InvalidStringLiteral => "invalid string literal".to_string(),
        }
    }

    fn level(&self) -> ReportLevel {
        match self.kind {
            ScannerErrorKind::IllegalCharacter(_) => {
                ReportLevel::Warning(WarningKind::ParsingWarning {
                    file_name: self.stream_name.clone(),
                    pos: self.location,
                    len: self.len,
                    source: self.line.clone(),
                })
            }
            _ => ReportLevel::Error(ErrorKind::ParsingError {
                file_name: self.stream_name.clone(),
                pos: self.location,
                len: self.len,
                source: self.line.clone(),
            }),
        }
    }

    fn help(&self) -> Option<String> {
        match self.kind {
            ScannerErrorKind::IllegalCharacter(_) => None,
            ScannerErrorKind::MalformedNumber(_) => <Option<String> as Clone>::clone(&self.help),
            ScannerErrorKind::InvalidCharLiteral => None,
            ScannerErrorKind::InvalidStringLiteral => None,
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        None
    }
}
