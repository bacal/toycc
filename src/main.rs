mod error;

use std::fmt::Display;
use std::fs::File;
use std::process::exit;

use crate::error::Error;
use toycc_argparser::Arguments;
use toycc_frontend::Parser;
use toycc_report::{Diagnostic, Report};
fn main() {
    let args = match Arguments::parse() {
        Ok(args) => args,
        Err(e) => {
            println!("{e}");
            exit(1)
        }
    };
    if args.help {
        Arguments::print_usage();
        exit(0);
    }
    if args.authors {
        Arguments::print_authors();
        exit(0)
    }
    if args.file_names.is_empty() {
        handle_error(Error::MissingInput);
    }
    let file = match File::open(&args.file_names[0]) {
        Ok(file) => file,
        Err(_) => handle_error(Error::FileNotFound(args.file_names[0].clone())),
    };

    let mut parser = Parser::new(&file, args.file_names[0].as_str(), args.debug, args.verbose);
    match parser.parse() {
        Ok(()) => {}
        Err(e) => handle_error(e),
    }
}

fn handle_error<T: Report + Diagnostic + Display>(error: T) -> ! {
    println!("{}", error);
    exit(1)
}
