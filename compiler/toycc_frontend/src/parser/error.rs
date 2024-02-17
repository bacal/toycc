use toycc_report::{Diagnostic, Report, ReportLevel};
use crate::scanner::error::ScannerError;

#[derive(Report)]
pub enum ParserError{
    ScannerError(ScannerError)
}

impl From<ScannerError> for ParserError{
    fn from(value: ScannerError) -> Self {
        Self::ScannerError(value)
    }
}

impl Diagnostic for ParserError{
    fn info(&self) -> String {
        match self{
            ParserError::ScannerError(s) => s.info()
        }
    }

    fn level(&self) -> ReportLevel {
        match self{
            ParserError::ScannerError(s) => s.level()
        }
    }

    fn help(&self) -> Option<String> {
        match self{
            ParserError::ScannerError(s) => s.help().to_owned()
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        match self{
            ParserError::ScannerError(s) => s.others()
        }
    }
}