extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Report)]
pub fn derive_report(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote!{
        use std::fmt;
        impl #impl_generics Report for #name #ty_generics #where_clause{
            fn message(&self) -> String{
                use colored::{Colorize, Color};
                use toycc_report::Level;

                match self.level(){
                    Level::Warning(_) => format!("{}","warning".bright_magenta().bold()),
                    Level::Error(e) => {
                        let level = "error".red().bold();
                        match e{
                            ErrorKind::ParsingError {file_name, pos, len, source} => {
                                let tail = match len {
                                    0 => "".to_owned(),
                                    len => format!("{}^\n{}{}"," ".repeat(pos.1-1), " ".repeat(pos.1-1),"~".repeat(len-1 as usize)).bright_green().to_string(),
                                };
                                format!("{}:{}:{}: {} {}\n{}\n{}",file_name, pos.0, pos.1, level, self.info(), source, tail)
                            },
                            ErrorKind::SimpleError(s) => format!("{}: {}: {}: {}","toycc".white().bold(), level, self.info().white().bold(), s.white().bold()),
                        }
                    },
                }
            }
        }
        impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f,"{}",self.message())
            }
        }
    };

    TokenStream::from(expanded)
}