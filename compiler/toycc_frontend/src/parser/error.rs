use crate::scanner::error::ScannerError;
use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};

#[derive(Report)]
pub enum ParserError {
    ScannerError(ScannerError),
    Generic,
    ExpectedType,
    ExpectedIdentifier,
    ExpectedFuncDef,
}

impl From<ScannerError> for ParserError {
    fn from(value: ScannerError) -> Self {
        Self::ScannerError(value)
    }
}

impl Diagnostic for ParserError {
    fn info(&self) -> String {
        match self {
            ParserError::ScannerError(s) => s.info(),
            _ => "".to_string(),
        }
    }

    fn level(&self) -> ReportLevel {
        match self {
            ParserError::ScannerError(s) => s.level(),
            _ => ReportLevel::Error(ErrorKind::NoHelpError),
        }
    }

    fn help(&self) -> Option<String> {
        match self {
            ParserError::ScannerError(s) => s.help().to_owned(),
            _ => None,
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        match self {
            ParserError::ScannerError(s) => s.others(),
            _ => None,
        }
    }
}
