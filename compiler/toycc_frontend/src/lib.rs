use std::io::{Read, Seek};
use toycc_report::{Diagnostic, Report};

mod scanner;
mod parser;
pub use parser::Parser;
