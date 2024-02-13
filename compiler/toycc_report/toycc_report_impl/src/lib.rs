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
                use toycc_report::ReportLevel;
                use toycc_report::ErrorKind;
                let mut buffer = String::new();
                let mut d = self.others();
                while let Some(c) = d{
                    buffer+=c.message().as_str();
                    buffer.push('\n');
                    d = c.others();
                }
                buffer + &*match self.level(){
                    ReportLevel::Warning(_) => format!("{}","warning".bright_magenta().bold()),
                    ReportLevel::Error(e) => {
                        let level = "error".red().bold();
                        match e{
                            ErrorKind::ParsingError {file_name, pos, len, source} => {
                                let tail = match len {
                                    0 => "".to_owned(),
                                    1 => format!("{}^"," ".repeat(pos.1-1)),
                                    len => format!("{}^\n{}{}"," ".repeat(pos.1-1), " ".repeat(pos.1-1),"~".repeat(len-1 as usize)).bright_green().to_string(),
                                };
                                let help = match self.help(){
                                    Some(msg) => format!("\n{}",msg),
                                    None => "".to_string(),
                                };
                                format!("{}:{}:{}: {}: {}\n{}\n{}{}",file_name.white().bold(),
                                    pos.0.to_string().white().bold(),
                                    pos.1.to_string().white().bold(),
                                    level,
                                    self.info().white().bold(),
                                    source, tail.bright_green(),
                                    help.bright_green())
                            },
                            ErrorKind::NoInfoError => format!("{}: {}: {}","toycc".white().bold(), level, self.info().white().bold()),
                            ErrorKind::SimpleError(s) => format!("{}: {}: {}: {}","toycc".white().bold(), level, self.info().white().bold(), s.white().bold()),
                        }
                    },
                }
            }
        }
        impl #impl_generics fmt::Display for #name #ty_generics #where_clause{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f,"{}",self.message())
            }
        }
    };

    TokenStream::from(expanded)
}