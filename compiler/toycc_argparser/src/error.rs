use crate::{OPTIONS, USAGE};
use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};

#[derive(Report, Debug, Eq, PartialEq)]
pub enum ArgumentParseError {
    UnknownArgument(String),
    ExtraPositional(String),
    InvalidDebug(u32),
    MissingValue(&'static str),
    Usage,
    Options,
}

impl Diagnostic for ArgumentParseError {
    fn info(&self) -> String {
        match self {
            Self::UnknownArgument(arg) => format!("unknown argument -{arg}"),
            Self::ExtraPositional(arg) => format!("unknown argument {arg}"),
            Self::InvalidDebug(num) => format!("invalid option for debug '{num}'"),
            Self::MissingValue(arg) => format!("missing value for -{arg}"),
            Self::Usage => "usage".to_string(),
            Self::Options => "options".to_string(),
        }
    }

    fn level(&self) -> ReportLevel {
        match self {
            Self::Usage | Self::Options => ReportLevel::Info,
            _ => ReportLevel::Error(ErrorKind::NoHelpError),
        }
    }

    fn help(&self) -> Option<String> {
        match self {
            Self::Usage => Some(USAGE.to_string()),
            Self::Options => Some(OPTIONS.to_string()),
            _ => None,
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        match self {
            Self::MissingValue(_) | Self::InvalidDebug(_) => Some(&Self::Usage),
            Self::Usage => Some(&Self::Options),
            _ => None,
        }
    }
}
