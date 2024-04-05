use toycc_report::{Diagnostic, Report, ReportLevel};

pub enum BackendError{
    SemanticError,
    CodeGenerationError,
}


#[derive(Report)]
pub struct SemanticError{
    kind: SemanticErrorKind,
}
impl SemanticError{
    pub fn new(kind: SemanticErrorKind) -> Self{
        Self{
            kind
        }
    }
}

pub enum SemanticErrorKind{
    UndeclaredIdentifier(String),
    MultipleBindings,
    IncompatibleTypes,
    InvalidReturn,
    DivisionByZero,
}

impl Diagnostic for SemanticError{
    fn info(&self) -> String {
        match &self.kind{
            SemanticErrorKind::UndeclaredIdentifier(id) => format!("undeclared identifier '\'{id}\'"),
            _ => Default::default(),
        }
    }

    fn level(&self) -> ReportLevel {
        todo!()
    }

    fn help(&self) -> Option<String> {
        todo!()
    }

    fn others(&self) -> Option<&dyn Report> {
        todo!()
    }
}