use crate::error::{SemanticError, SemanticErrorKind};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use toycc_frontend::Type;

/// Symbol names in table are mangled to avoid collisions
#[derive(Debug, Default, Clone)]
pub struct SymbolTable<'a> {
    table: HashMap<&'a str, Symbol>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Variable(String, Type, usize),
    Function(Function),
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Variable(name, t, _) => {
                write!(f, "[Variable] Name: {:<10}\tType: {:>7}", name, t)
            }
            Symbol::Function(function) => {
                write!(
                    f,
                    "[Function] name: {:<10}\tReturn Type: {:<4}\tArgs: {:<20}",
                    function.name,
                    function.return_type,
                    function.arguments.join(","),
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<String>,
    pub body: Vec<String>,
    pub return_type: Type,
}

impl Function {
    pub fn new(name: String, arguments: Vec<String>, body: Vec<String>, return_type: Type) -> Self {
        Self {
            name,
            arguments,
            body,
            return_type,
        }
    }
}

impl<'a> SymbolTable<'a> {
    pub fn insert(&mut self, name: &'a str, symbol: Symbol) -> Result<&Symbol, Box<SemanticError>> {
        match self.table.insert(name, symbol) {
            Some(_) => Err(Box::new(
                self.create_error(SemanticErrorKind::MultipleBindings(name.to_string())),
            )),
            None => Ok(self.table.get(name).unwrap()),
        }
    }
    pub fn find(&mut self, name: &str) -> Option<&Symbol> {
        self.table.get(name)
    }

    fn create_error(&mut self, kind: SemanticErrorKind) -> SemanticError {
        SemanticError::new(kind)
    }
}

impl<'a> Display for SymbolTable<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let functions = self
            .table
            .iter()
            .map(|e| e.1)
            .filter(|e| matches!(e, Symbol::Function(_)))
            .join("\n");
        let variables = self
            .table
            .iter()
            .map(|e| e.1)
            .filter(|e| matches!(e, Symbol::Variable(..)))
            .join("\n");

        write!(
            f,
            "Symbol Table\n------------\n{}\n{}",
            functions, variables
        )
    }
}
