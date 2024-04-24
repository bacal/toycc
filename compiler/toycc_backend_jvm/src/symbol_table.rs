use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use toycc_frontend::Type;
use crate::error::{SemanticError, SemanticErrorKind};

/// Symbol names in table are mangled to avoid collisions
#[derive(Debug, Default, Clone)]
pub struct SymbolTable<'a>{
    table: HashMap<&'a str, Symbol>,
}
#[derive(Debug, Clone)]
pub enum Symbol{
    Variable(Type, usize),
    Function(Function),
}

#[derive(Debug, Clone)]
pub struct Function{
    pub name: String,
    pub arguments: Vec<String>,
    pub body: Vec<String>,
    pub return_type: Type,
}

impl Function{
    pub fn new(name: String, arguments: Vec<String>, body: Vec<String>, return_type: Type) -> Self{
        Self{
            name,
            arguments,
            body,
            return_type,
        }
    }
}


impl<'a> SymbolTable<'a>{
    pub fn insert(&mut self, name: &'a str, symbol: Symbol) -> Result<&Symbol, Box<SemanticError>>{
        match self.table.insert(name,symbol){
            Some(_) => Err(Box::new(self.create_error(SemanticErrorKind::MultipleBindings(name.to_string())))),
            None => Ok(self.table.get(name).unwrap())
        }
    }
    pub fn find(&mut self, name: &str) -> Option<&Symbol>{
        self.table.get(name)
    }

    pub fn len(&mut self) -> usize{
        self.table.len()
    }
    fn create_error(&mut self, kind:  SemanticErrorKind) -> SemanticError{
        SemanticError::new(kind)
    }
}


impl<'a> Display for SymbolTable<'a>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod test{
    use super::*;
}