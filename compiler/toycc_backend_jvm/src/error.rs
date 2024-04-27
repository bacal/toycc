use toycc_report::{Diagnostic, ErrorKind, Report, ReportLevel};

#[derive(Report, Debug)]
pub struct SemanticError {
    kind: SemanticErrorKind,
}
impl SemanticError {
    pub fn new(kind: SemanticErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum SemanticErrorKind {
    UndeclaredIdentifier(String),
    UndeclaredFunction(String),
    MultipleBindings(String),
    IncompatibleTypes,
    InvalidReturn(String, String),
    DivisionByZero,
    MissingMain,
    ExpectedFunction,
    ExpectedIdentifier,
    MissingReturn,
}

impl Diagnostic for SemanticError {
    fn info(&self) -> String {
        match &self.kind {
            SemanticErrorKind::UndeclaredIdentifier(id) => {
                format!("undeclared identifier \'{id}\'")
            }
            SemanticErrorKind::MissingMain => "missing main function".to_owned(),
            SemanticErrorKind::UndeclaredFunction(ud) => format!("undeclared function {ud}"),
            SemanticErrorKind::MultipleBindings(id) => format!("redeclaration of identifier {id}"),
            SemanticErrorKind::IncompatibleTypes => "incompatible types".to_owned(),
            SemanticErrorKind::InvalidReturn(expected, actual) => {
                format!("incompatible return types: expected: {expected} actual: {actual}")
            }
            SemanticErrorKind::DivisionByZero => "illegal division by 0".to_owned(),
            SemanticErrorKind::ExpectedFunction => "expected function declaration".to_owned(),
            SemanticErrorKind::ExpectedIdentifier => "expected identifier".to_owned(),
            SemanticErrorKind::MissingReturn => "missing return".to_owned(),
        }
    }

    fn level(&self) -> ReportLevel {
        ReportLevel::Error(ErrorKind::NoHelpError)
    }

    fn help(&self) -> Option<String> {
        None
    }

    fn others(&self) -> Option<&dyn Report> {
        None
    }
}
