use std::collections::HashMap;
use crate::error::{SemanticError, SemanticErrorKind};

/// Symbol names in table are mangled to avoid collisions
#[derive(Debug, Default)]
pub struct SymbolTable<'a>{
    table: HashMap<&'a str, Symbol>,
    scope: Vec<String>,
}
#[derive(Debug)]
pub enum Symbol{
    Variable,
    Label,
    Offset,
}

impl<'a> SymbolTable<'a>{
    pub fn insert(&mut self, name: &'a str, symbol: Symbol) -> Result<&Symbol, SemanticError>{
        match self.table.insert(name,symbol){
            Some(_) => Err(self.create_error(SemanticErrorKind::MultipleBindings)),
            None => Ok(self.table.get(name).unwrap())
        }
    }
    pub fn find(&mut self, name: &str) -> Option<&Symbol>{
        self.table.get(name)
    }

    fn create_error(&mut self, kind:  SemanticErrorKind) -> SemanticError{
        SemanticError::new(kind)
    }
}

#[cfg(test)]
mod test{
    use super::*;
}