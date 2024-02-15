mod error;

use std::fmt::{Display};
use std::fs::File;
use std::process::exit;
use std::sync::Arc;

use toycc_argparser::Arguments;
use toycc_parser::Parser;
use toycc_report::{Diagnostic, Report};
use crate::error::Error;
fn main(){
    let args = match Arguments::parse(){
        Ok(args) => args,
        Err(e) => { println!("{e}"); exit(1)},
    };
    if args.help{
        Arguments::print_usage();
        exit(0);
    }
    if args.file_names.is_empty(){
        handle_error(Error::MissingInput)
    }
    let file = match File::open(&args.file_names[0]){
        Ok(file) => file,
        Err(_) => {
            handle_error(Error::FileNotFound(args.file_names[0].clone()));
            exit(1);
        },
    };

    let mut parser = Parser::new(Arc::new(file),args.file_names[0].clone(),args.debug);
    match parser.parse(){
        Ok(()) => {},
        Err(e) => handle_error(e),
    }
}

fn handle_error<T: Report + Diagnostic + Display>(error: T){
    println!("{}",error);
}