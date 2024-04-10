use itertools::Itertools;
use toycc_frontend::ast::{Definition, FuncDef, Program, Statement, VarDef};
use crate::error::{SemanticError, SemanticErrorKind};
use crate::symbol_table::{Symbol, SymbolTable};

#[derive(Default)]
pub struct SemanticAnalyzer<'a> {
    symbol_table: Vec<SymbolTable<'a>>,
}

impl<'a> SemanticAnalyzer<'a>{
    pub fn new() -> Self{
        Self{
            symbol_table: vec![SymbolTable::default()]
        }
    }
    pub fn analyze_program(&mut self, program: &'a Program) -> Result<Vec<String>, Box<SemanticError>>{
        let x: Vec<_> = program.definitions
            .iter()
            .map(|def| self.analyze_definition(def))
            .fold_ok(vec![], |mut acc, mut e| { acc.append(&mut e); acc })?;

        // if self.symbol_table[0].find("main").is_none(){
        //     return Err(Box::new(SemanticError::new(SemanticErrorKind::MissingMain)));
        // }
        println!("{:?}",x);


        Ok(x)
    }

    fn analyze_definition(&mut self, definition: &'a Definition) -> Result<Vec<String>, Box<SemanticError>>{
        match definition{
            Definition::FuncDef(func_def) => self.analyze_func_def(func_def),
            Definition::VarDef(var_def) => self.analyze_var_def(var_def),
        }?;
        Ok(vec![])
    }

    fn analyze_func_def(&mut self, func_def: &'a FuncDef) -> Result<(), Box<SemanticError>>{
        self.push_scope();
        for var in &func_def.var_def{
            self.analyze_var_def(&var)?;
        }
        self.symbol_table.iter_mut()
            .next_back()
            .unwrap()
            .insert(func_def.identifier.as_str(), Symbol::Function(func_def.toyc_type.clone()))?;

        self.analyze_statement(&func_def.statement)?;

        println!("{:#?}", self.symbol_table);
        Ok(())
    }
    fn analyze_var_def(&mut self, var_def: &'a VarDef) -> Result<(), Box<SemanticError>>{
        for id in &var_def.identifiers{
            self.symbol_table.iter_mut()
                .next_back()
                .unwrap()
                .insert(id.as_str(),Symbol::Variable(var_def.toyc_type.clone()))?;
        }

        Ok(())
    }

    fn push_scope(&mut self){
        self.symbol_table.push(self.symbol_table.iter().next_back().unwrap().clone())
    }

    fn analyze_statement(&mut self, statement: &'a Statement) -> Result<(), Box<SemanticError>>{
        match statement{
            Statement::Expression(expr) => {}
            Statement::Break => {}
            Statement::BlockState(var_defs, statements) => {
                for var in var_defs{
                    self.analyze_var_def(var)?;
                }
                for statement in statements{
                    self.analyze_statement(statement)?;
                }
            }
            Statement::IfState(expr, statement, other_statements) => {
                self.analyze_statement(statement)?;
                for statement in other_statements{
                    self.analyze_statement(statement)?;
                }
            }
            Statement::NullState => {}
            Statement::ReturnState(_) => {}
            Statement::WhileState(_, _) => {}
            Statement::ReadState(_, _) => {}
            Statement::WriteState(_, _) => {}
            Statement::NewLineState => {}
        }
        Ok(())
    }

    fn pop_scope(&mut self){
        self.symbol_table.pop();
    }

}

#[cfg(test)]
pub mod test{
    use std::io::Cursor;
    use super::*;
    #[test]
    fn test_valid_program(){
        let program = toycc_frontend::Parser::new(
            Cursor::new("int a; int b; int main(){int c; int d;}"),
            "test.tc",
            Some(2),
            false).parse().expect("failed to parse");
        println!("{:?}",program);
        let mut analyzer = SemanticAnalyzer::new();
        let c = analyzer.analyze_program(&program);
        assert!(c.is_ok());
    }
}