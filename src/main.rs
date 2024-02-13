mod error;

use std::fs::File;
use std::string::ToString;
use clap::Parser;
use toycc_report::{Diagnostic, ErrorKind, Level, Report};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about, after_help = "ToyC Compiler for EGRE591 (SPRING 2024)")]
struct Arguments{
    #[arg(value_name = "INPUT", required = true)]
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
    FileNotFound(String),
}

impl Diagnostic for Error{

    fn info(&self) -> &str {
        match self{
            Error::FileNotFound(name) => name.as_str(),
        }
    }

    fn level(&self) -> Level {
        match self{
           Self::FileNotFound(_) => Level::Error(ErrorKind::SimpleError("file not found".to_string())),
        }

    }

    fn help(&self) -> Option<&str> {
        None
    }

    fn others(&self) -> Option<&dyn Diagnostic> {
        None
    }
}
fn main(){
    let args = Arguments::parse();
    let file = match File::open(&args.file_names[0]){
        Ok(file) => {}
        Err(_) => println!("{}",Error::FileNotFound(args.file_names[0].clone()))
    };
}