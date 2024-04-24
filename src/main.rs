mod error;

use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::Path;
use std::process::exit;

use crate::error::Error;
use toycc_argparser::Arguments;
use toycc_backend_jvm::semantic_analyzer::SemanticAnalyzer;
use toycc_frontend::Parser;
use toycc_report::{Diagnostic, Report};
fn main() {
    let args = match Arguments::parse() {
        Ok(args) => args,
        Err(e) => handle_error(e),
    };
    if args.help {
        Arguments::print_usage();
        exit(0);
    }
    if args.authors {
        Arguments::print_authors();
        exit(0)
    }
    if args.file_name.is_none() {
        handle_error(Error::MissingInput);
    }

    let debug = match args.verbose {
        true => Some(0),
        false => args.debug,
    };

    let file = match File::open(args.file_name.as_ref().unwrap()) {
        Ok(file) => file,
        Err(_) => handle_error(Error::FileNotFound(args.file_name.unwrap())),
    };
    let path = Path::new(OsStr::new(args.file_name.as_ref().unwrap()));

    let mut parser = Parser::new(&file, args.file_name.as_ref().unwrap().as_str(), debug);

    let parsed_program = parser.parse().unwrap_or_else(|e| handle_error(*e));
    if args.dump_ast || args.verbose {
        println!("{parsed_program}");
    }

    let file_name = args
        .output
        .unwrap_or(path.file_stem().unwrap().to_string_lossy().to_string());
    let class_name = args.class.unwrap_or(file_name.clone());
    let jasmin_program = SemanticAnalyzer::new(class_name.as_str(), args.dump_sym)
        .analyze_program(&parsed_program)
        .unwrap_or_else(|e| handle_error(*e));

    if args.dump_cgn || args.verbose {
        println!("{jasmin_program}");
    }

    let mut output_file = File::create(format!("{file_name}.j")).unwrap();
    output_file
        .write_all(jasmin_program.as_bytes())
        .expect("failed to write to file");
}

fn handle_error<T: Report + Diagnostic + Display>(error: T) -> ! {
    let _ = stdout().flush();
    println!("{}", error);
    exit(1)
}
