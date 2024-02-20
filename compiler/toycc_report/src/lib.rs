pub enum ErrorKind {
    ParsingError {
        file_name: String,
        pos: (usize, usize),
        len: usize,
        source: Option<String>,
    },
    SimpleError(String),
    NoHelpError,
}

pub enum WarningKind {}

pub enum ReportLevel {
    Warning(WarningKind),
    Error(ErrorKind),
    Info,
}

pub trait Diagnostic {
    fn info(&self) -> String;
    fn level(&self) -> ReportLevel;
    fn help(&self) -> Option<String>;
    fn others(&self) -> Option<&dyn Report>;
}

pub trait Report: Diagnostic {
    fn message(&self) -> String;
}

pub use toycc_report_impl::*;
