use std::collections::HashMap;
use toycc_frontend::Type;
use crate::error::{SemanticError, SemanticErrorKind};

/// Symbol names in table are mangled to avoid collisions
#[derive(Debug, Default, Clone)]
pub struct SymbolTable<'a>{
    table: HashMap<&'a str, Symbol>,
}
#[derive(Debug, Clone)]
pub enum Symbol{
    Variable(Type),
    Function(Type),
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