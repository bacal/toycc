mod error;

use std::fmt::Display;
use std::fs::File;
use std::process::exit;
use clap::Parser as Clap_Parser;
use toycc_parser::Parser;
use toycc_report::{Diagnostic, Report};
use crate::error::Error;

#[derive(Clap_Parser, Debug)]
#[command(author, version, about, long_about, after_help = "ToyC Compiler for EGRE591 (SPRING 2024)")]
struct Arguments {
    #[arg(value_name = "INPUT", required = false)]
    file_names: Vec<String>,

    #[arg(
    short,
    long,
    value_name = "level",
    help = "Display messages that aid in tracing the compilation process.\
              \n0 - all messages \
              \n1 - scanner messages only",
    default_value = None
    )]
    debug: Option<i32>,

    #[arg(short, long, help = "Display all information")]
    verbose: bool,
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
    let mut parser = Parser::new(file.unwrap(),args.file_names[0].clone(),args.debug);
    match parser.parse(){
        Ok(()) => {},
        Err(e) => handle_error(e),
    }
}

fn handle_error<T: Report + Diagnostic + Display>(error: T){
    println!("{}",error);
    exit(1);
}