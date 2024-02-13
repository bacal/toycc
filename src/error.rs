use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};

#[derive(Report)]
pub enum Error{
    MissingInput,
    FileNotFound(String),
    Nothing,
}

impl Diagnostic for Error{

    fn info(&self) -> String {
        match self{
            Error::MissingInput => "no input files".to_string(),
            Error::FileNotFound(name) => name.clone(),
            _ => "".to_string(),
        }
    }

    fn level(&self) -> ReportLevel {
        match self{
            Self::MissingInput => ReportLevel::Error(ErrorKind::NoInfoError),
            Self::FileNotFound(_) => ReportLevel::Error(ErrorKind::SimpleError("file not found".to_string())),
            _ => ReportLevel::Error(ErrorKind::SimpleError("blah".to_string())),
        }

    }

    fn help(&self) -> Option<&str> {
        None
    }

    fn others(&self) -> Option<&dyn Report> {
        None
    }
}