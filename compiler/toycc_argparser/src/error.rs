use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};
const DEBUG_USAGE : &str = r"-debug <level>  0 - all messages
                       1 - scanner messages only";
#[derive(Report)]
pub enum ArgumentParseError{
    UnknownArgument(String),
    InvalidDebug(u32),
    MissingValue,
    Usage(&'static str),
}

impl Diagnostic for ArgumentParseError{
    fn info(&self) -> String {
        match self{
            Self::UnknownArgument(arg) => format!("unknown argument -{arg}"),
            Self::InvalidDebug(num) => format!("invalid option for debug '{num}'"),
            Self::MissingValue => "missing value for -debug".to_string(),
            Self::Usage(_)=> "usage".to_string()
        }
    }

    fn level(&self) -> ReportLevel {
        match self{
            Self::Usage(_) => ReportLevel::Info,
            _ => ReportLevel::Error(ErrorKind::NoInfoError)
        }
    }

    fn help(&self) -> Option<String> {
        match self{
            Self::Usage(usage) => Some(usage.to_string()),
            _ => None,
        }
    }

    fn others(&self) -> Option<&dyn Report> {
        match self{
            Self::InvalidDebug(_) => Some(&Self::Usage(DEBUG_USAGE)),
            _ => None
        }
    }
}