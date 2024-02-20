extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Report)]
pub fn derive_report(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

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

                buffer += &*match self.level(){
                    ReportLevel::Info => format!("{}: {}",self.info().white().bold(), self.help().unwrap_or_default()),
                    ReportLevel::Warning(_) => format!("toycc: {}","warning".bright_magenta().bold()),
                    ReportLevel::Error(e) => {
                        let level = "error".red().bold();
                        match e{
                            ErrorKind::ParsingError {file_name, pos, len, source} => {
                                let spaces = " ".repeat(pos.1-1);
                                let help = match self.help(){
                                    Some(s) => format!("\n{spaces}{s}"),
                                    None => "".to_string()
                                };
                                let tail = match len {
                                    0 => "".to_owned(),
                                    1 => format!("\n{}^",spaces),
                                    len => format!("\n{}^\n{}{}",spaces,spaces,"~".repeat(len-1 as usize)).bright_green().to_string(),
                                };
                                let source = match source{
                                    Some(source) => format!("\n{source}"),
                                    None => "".to_owned(),
                                };
                                format!("{}:{}:{}: {}: {}{}{}{}{}",
                                    file_name.white().bold(),
                                    pos.0.to_string().white().bold(),
                                    pos.1.to_string().white().bold(),
                                    level,
                                    self.info().white().bold(),
                                    source,
                                    tail.bright_green(),
                                    spaces,
                                    help.bright_green())
                            },
                            ErrorKind::NoHelpError => format!("{}: {}: {}","toycc".white().bold(), level, self.info().white().bold()),
                            ErrorKind::SimpleError(s) => format!("{}: {}: {}: {}","toycc".white().bold(), level, self.info().white().bold(), s.white().bold()),
                        }
                    },
                };
                while let Some(c) = d{
                    buffer.push('\n');
                    buffer+=c.message().as_str();
                    d = c.others();
                }
                buffer
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