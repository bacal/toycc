mod error;

use std::fs::File;
use std::io::Cursor;
use std::process::exit;
use std::string::ToString;
use clap::Parser as Clap_Parser;
use toycc_parser::Parser;
use toycc_report::{Diagnostic, ErrorKind, ReportLevel, Report};

#[derive(Clap_Parser, Debug)]
#[command(author, version, about, long_about, after_help = "ToyC Compiler for EGRE591 (SPRING 2024)")]
struct Arguments{
    #[arg(value_name = "INPUT", required = false)]
    file_names: Vec<String>,

    #[arg(
    short,
    long,
    value_name = "level",
    help="Display messages that aid in tracing the compilation process.\
              \n0 - all messages \
              \n1 - scanner messages only",
    default_value = None
    )]
    debug: Option<i32>,

    #[arg(short, long, help="Display all information")]
    verbose: bool,
}
#[derive(Report)]
enum Error{
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
fn main(){
    let args = Arguments::parse();
    if args.file_names.is_empty(){
        handle_error(Error::MissingInput)
    }
    let file = match File::open(&args.file_names[0]){
        Ok(file) => Some(file),
        Err(_) => {
            handle_error(Error::FileNotFound(args.file_names[0].clone()));
            None
        },
    };
    let mut parser = Parser::new(Cursor::new("112E+0.1"), "file.tc".to_string(), None);
    match parser.parse(){
        Ok(()) => {},
        Err(e) => handle_error(e),
    }
    // let parser = Parser::new(file.unwrap(),args.file_names[0].clone(),args.debug);
}

fn handle_error<T: Report + Diagnostic + fmt::Display>(error: T){
    println!("{}",error);
    exit(1);
}