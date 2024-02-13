pub enum ErrorKind{
    ParsingError{
        file_name: String,
        pos: (usize,usize),
        len: usize,
        source: String,
    },
    SimpleError(String),
}

pub enum WarningKind{

}

pub enum Level{
    Warning(WarningKind),
    Error(ErrorKind)
}

pub trait Diagnostic{
    fn info(&self) -> &str;
    fn level(&self) -> Level;
    fn help(&self) -> Option<&str>;
    fn others(&self) -> Option<&dyn Diagnostic>;
}


pub trait Report: Diagnostic{
    fn message(&self) -> String;
}

pub use toycc_report_impl::*;