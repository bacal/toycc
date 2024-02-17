use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};

#[derive(Report)]
pub enum Error{
    MissingInput,
    FileNotFound(String),
}

impl Diagnostic for Error{

    fn info(&self) -> String {
        match self{
            Error::MissingInput => "no input files".to_string(),
            Error::FileNotFound(_) => "no such file or directory".to_string(),
        }
    }

    fn level(&self) -> ReportLevel {
        match self{
            Self::MissingInput => ReportLevel::Error(ErrorKind::NoInfoError),
            Self::FileNotFound(name) => ReportLevel::Error(ErrorKind::SimpleError(format!("'{name}'"))),
        }

    }

    fn help(&self) -> Option<String> {
        None
    }

    fn others(&self) -> Option<&dyn Report> {
        match self{
            Self::FileNotFound(_) => Some(&Self::MissingInput),
            _ => None,
        }
    }
}