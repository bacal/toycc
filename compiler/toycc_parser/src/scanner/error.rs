use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};

#[derive(Debug, PartialEq)]
pub enum ScannerErrorKind{
    IllegalCharacter(char),
    MalformedNumber,
}

#[derive(Debug, Report, PartialEq)]
pub struct ScannerError{
    kind: ScannerErrorKind,
    line: String,
    location: (usize, usize),
    len: usize,
    stream_name: String,
    help: Option<String>,
}

impl ScannerError{
    pub fn new(kind: ScannerErrorKind, line: String,
               location: (usize, usize), len: usize,
               stream_name: String, help: Option<String>) -> Self{
        Self{
            kind,
            line,
            location,
            len,
            stream_name,
            help,
        }
    }
}


impl Diagnostic for ScannerError{
    fn info(&self) -> String {
        match &self.kind{
            ScannerErrorKind::IllegalCharacter(c) => format!("illegal character: '{c}'"),
            ScannerErrorKind::MalformedNumber =>  <Option<String> as Clone>::clone(&(&self.help)).unwrap_or(String::new())
        }
    }

    fn level(&self) -> ReportLevel {
        ReportLevel::Error(ErrorKind::ParsingError {
            file_name: self.stream_name.clone(),
            pos: self.location,
            len: self.len,
            source: self.line.clone(),
        })
    }

    fn help(&self) -> Option<&str> {
        match self.kind{
            ScannerErrorKind::IllegalCharacter(_) => None,
            ScannerErrorKind::MalformedNumber => None,
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        None
    }
}
