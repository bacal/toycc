use std::fs::File;
use clap::Parser;

use toyc_frontend::ToyCFrontend as FrontEnd;
use toycc_api::frontend::prelude::*;
use toycc_api::TccFrontEnd;

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


fn main(){
    let args = Arguments::parse();
    let f = File::open(&args.file_names[0].as_str()).expect("file not found");
    // let mut frontend = FrontEnd::new();
    // frontend.load_data(f);
}