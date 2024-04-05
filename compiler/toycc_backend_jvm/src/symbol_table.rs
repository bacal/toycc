use std::collections::HashMap;
use crate::error::{SemanticError, SemanticErrorKind};

/// Symbol names in table are mangled to avoid collisions
#[derive(Debug, Default)]
pub struct SymbolTable{
    table: HashMap<String, Symbol>,
    scope: Vec<String>,
}
#[derive(Debug)]
pub enum Symbol{
    Variable,
    Label,
    Offset,
}

impl SymbolTable{
    pub fn insert(&mut self, name: &str, symbol: Symbol) -> Result<&Symbol, SemanticError>{
        match self.table.insert(name,symbol){
            Some(_) => Err(self.create_error(SemanticErrorKind::MultipleBindings)),
            None => Ok(self.table.get(name).unwrap())
        }
    }
    pub fn find(&mut self, name: &str) -> Option<&Symbol>{
        self.table.get(name)
    }
    fn mangle_name(&mut self, name: &str) -> String{
        format!("{}_{}", self.scope.iter().next_back().expect("missing scope"), name)
    }

    fn push_scope(&mut self, scope: String){
        self.scope.push(scope);
    }

    fn pop_scope(&mut self){
        self.table.retain(|key,_| !key.starts_with(self.scope.iter().next_back().unwrap()));
        self.scope.pop();
    }

    fn create_error(&mut self, kind:  SemanticErrorKind) -> SemanticError{
        SemanticError::new(kind)
    }
}

#[cfg(test)]
mod test{
    use super::*;
}