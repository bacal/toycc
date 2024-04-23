use itertools::Itertools;
use toycc_frontend::ast::{Definition, Expression, FuncDef, Operator, Program, Statement, VarDef};
use toycc_frontend::Type;
use crate::error::{SemanticError, SemanticErrorKind};
use crate::symbol_table::{Function, Symbol, SymbolTable};


const CLASS_INIT_HEADER : &str =
r#"
.method public <init> ()V
    aload_0
    invokespecial java/lang/Object/<init>()V
    return
.end_method

"#;

#[derive(Default)]
pub struct SemanticAnalyzer<'a> {
    program_name: &'a str,
    symbol_table: Vec<SymbolTable<'a>>,
    if_count: usize,
    while_count: usize,
}

impl<'a> SemanticAnalyzer<'a>{
    pub fn new() -> Self{
        Self{
            if_count: 0,
            while_count: 0,
            program_name: "",
            symbol_table: vec![SymbolTable::default();1]
        }
    }
    pub fn analyze_program(&mut self, program: &'a Program, name: &'a str) -> Result<String, Box<SemanticError>>{
        self.program_name = name;
        let mut jasmin_program = format!(".class public {name}\n.super java/lang/Object{}\n",CLASS_INIT_HEADER.to_string());
        let mut x: Vec<_> = program.definitions
            .iter()
            .map(|def| self.analyze_definition(def))
            .fold_ok(vec![], |mut acc, mut e| { acc.append(&mut e); acc })?;
        println!("{:?}",x);
        // if self.symbol_table[0].find("main").is_none(){
        //     return Err(Box::new(SemanticError::new(SemanticErrorKind::MissingMain)));
        // }
        jasmin_program += x.join("\t\n").as_str();

        Ok(jasmin_program)
    }

    fn analyze_definition(&mut self, definition: &'a Definition) -> Result<Vec<String>, Box<SemanticError>>{
        match definition{
            Definition::FuncDef(func_def) => self.analyze_func_def(func_def),
            Definition::VarDef(var_def) => self.analyze_var_def(var_def),
        }
    }

    fn analyze_func_def(&mut self, func_def: &'a FuncDef) -> Result<Vec<String>, Box<SemanticError>>{
        let mut instructions = vec![];
        let return_type = match func_def.toyc_type{
            Type::Int => "I",
            Type::Char => "C",
        };
        self.push_scope();

        func_def.var_def.iter().map(|def| self.analyze_var_def(&def)).fold_ok(0, |acc, f| {
            f;
            0
        })?;

        let args: Vec<_> = func_def.var_def.iter()
            .map(|arg|{
                match arg.toyc_type{
                    Type::Int => "I".to_string(),
                    Type::Char => "C".to_string(),
                }
            })
            .collect();

        let mut body = self.analyze_statement(&func_def.statement)?;
        println!("{:?}",&body);
        self.pop_scope();
        let function = Function::new(func_def.identifier.clone(),
                                     args.clone(),
                                     body.clone(),
                                     func_def.toyc_type.clone());
        println!("{:?}",&function);
        self.insert_symbol(func_def.identifier.as_str(), Symbol::Function(function))?;

        body.iter_mut()
            .filter(|f| !f.starts_with("."))
            .for_each(|mut f| f.insert(0,'\t'));

        instructions.push(format!(".method public static {}({}){}\n{}\n.end_method\n",
                              func_def.identifier,
                              args.join(""),
                              return_type,
                              body.join("\n")));
        Ok(instructions)
    }
    fn analyze_var_def(&mut self, var_def: &'a VarDef) -> Result<Vec<String>, Box<SemanticError>>{

        for id in &var_def.identifiers{
            let pos = self.symbol_table.iter_mut().next_back().unwrap().len();
            self.insert_symbol(id.as_str(), Symbol::Variable(var_def.toyc_type.clone(), pos+1))?;
        }

        Ok(vec![])
    }

    fn push_scope(&mut self){
        self.symbol_table.push(self.symbol_table.iter().next_back().unwrap().clone())
    }

    fn analyze_statement(&mut self, statement: &'a Statement) -> Result<Vec<String>, Box<SemanticError>>{
        let mut instructions = vec![];
        match statement{
            Statement::Expression(expr) => instructions.append(&mut self.analyze_expression(expr)?),
            Statement::Break => instructions.push("ret".to_string()),
            Statement::BlockState(var_defs, statements) => {
                instructions.append(
                    &mut var_defs.iter()
                        .map(|a| self.analyze_var_def(a))
                        .collect::<Result<Vec<_>, Box<SemanticError>>>()?
                        .iter().flatten()
                        .map(Clone::clone)
                        .collect::<Vec<_>>()
                );
                instructions.append(
                    &mut statements.iter()
                        .map(|s| self.analyze_statement(s))
                        .collect::<Result<Vec<_>, Box<SemanticError>>>()?
                        .iter().flatten()
                        .map(Clone::clone)
                        .collect::<Vec<_>>()
                );
            }
            Statement::IfState(expr, statement, else_stmt) => {
                self.if_count+=1;
                let then_label = format!(".CT{}",self.if_count);
                let end_label = format!(".CE{}",self.if_count);
                match expr{
                    Expression::Expr(op, lhs, rhs) => {
                        instructions.append(&mut self.analyze_expression(lhs)?);
                        instructions.append(&mut self.analyze_expression(rhs)?);
                        let op = match op {
                            Operator::LessEqual => "ifle",
                            Operator::LessThan => "iflt",
                            Operator::GreaterEqual => "ifge",
                            Operator::GreaterThan => "ifgt",
                            Operator::Equal => "ifeq",
                            Operator::NotEqual => "ifne",
                            _ => todo!("Add Error Handling"),
                        };
                        instructions.push(format!("{op} {then_label}"));
                    },
                    _ => {
                        instructions.append(&mut self.analyze_expression(expr)?);
                        instructions.push("ldc 1".to_string());
                        instructions.push(format!("ifeq {then_label}"));

                    },
                };
                instructions.push(format!("jump {end_label}"));
                instructions.push(format!("{then_label}:"));
                instructions.append(&mut self.analyze_statement(statement)?);

                instructions.push(format!("{end_label}:"));
                if let Some(else_statement) = else_stmt.as_ref(){
                    instructions.append(&mut self.analyze_statement(else_statement)?);
                }
            }
            Statement::NullState => instructions.push("nop".to_string()),
            Statement::ReturnState(arg) => {
                match arg{
                    Some(arg) => {
                        instructions.append(&mut self.analyze_expression(&arg)?);
                        instructions.push("ireturn".to_string());
                    }
                    None => instructions.push("return".to_string())
                }

            }
            Statement::WhileState(expr, statement) => {

            }
            Statement::ReadState(_, _) => {}
            Statement::WriteState(_, _) => {}
            Statement::NewLineState => {}
        }
        Ok(instructions)
    }

    fn analyze_expression(&mut self, expression: &'a Expression) -> Result<Vec<String>, Box<SemanticError>>{
        let mut instructions = vec![];
        match expression{
            Expression::Number(num) => {
                instructions.push(format!("ldc {num}"));
            }
            Expression::Identifier(id) => {
                match self.get_symbol(id)?{
                    Symbol::Variable(_, num) => instructions.push(format!("iload {num}")),
                    _ => return Err(Box::new(SemanticError::new(SemanticErrorKind::ExpectedIdentifier)))
                }
            }
            Expression::CharLiteral(c) => {
                if let Some(c) = c{
                   instructions.push(format!("bipush {}", *c as u32));
                }
            }
            Expression::StringLiteral(s) => {
                instructions.push(format!("ldc {s}"));
            }

            Expression::FuncCall(name, arguments) => {
                let program_name = self.program_name;
                instructions.append(&mut arguments.iter()
                    .map(|a| self.analyze_expression(a))
                    .collect::<Result<Vec<_>, Box<SemanticError>>>()?
                    .iter()
                    .flatten()
                    .map(Clone::clone)
                    .collect::<Vec<_>>());

                if let Symbol::Function(func) = self.get_symbol(name)?{
                    let call = format!("invokestatic {}/{name}({})",program_name,func.arguments.clone().join(""));
                    instructions.push(call);
                }
                else{
                    return Err(Box::new(SemanticError::new(SemanticErrorKind::UndeclaredFunction(name.clone()))))
                }
            }
            Expression::Expr(op, expra, exprb) => {
                instructions.append(&mut self.analyze_expression(exprb)?);
                if *op == Operator::Assign{
                    match expra.as_ref(){
                        Expression::Identifier(id) => {
                            match self.get_symbol(id)?{
                                Symbol::Variable(_, num) => {
                                    instructions.push(format!("istore {num}"));
                                }
                                _ => return Err(Box::new(SemanticError::new(SemanticErrorKind::UndeclaredIdentifier(id.clone())))),
                            }
                        }
                        _ => return Err(Box::new(SemanticError::new(SemanticErrorKind::ExpectedIdentifier))),
                    }
                }
                else {
                    instructions.append(&mut self.analyze_expression(expra)?);
                    let op = match op{
                        Operator::Plus => "iadd".to_owned(),
                        Operator::Minus => "isub".to_owned(),
                        Operator::Multiply => "imul".to_owned(),
                        Operator::Divide => "idiv".to_owned(),
                        Operator::Modulo => "irem".to_owned(),
                        Operator::Or => "ior".to_owned(),
                        Operator::And => "iand".to_owned(),
                        _ => "".to_owned(),
                    };
                    instructions.push(op);
                }

            }
            Expression::Not(expr) => {
                let expr = self.analyze_expression(expr)?;
                let inst1 = "iconst_m1";
                let inst = "ixor"; //not operation
            }
            Expression::Minus(expr) => {
                let expr = self.analyze_expression(expr)?;
                let inst = "inot";
            }
        }

        Ok(instructions)
    }

    fn pop_scope(&mut self){
        self.symbol_table.pop();
    }

    fn get_symbol(&mut self, name: &'a str) -> Result<&Symbol, Box<SemanticError>>{
        Ok(self.symbol_table.iter_mut().next_back().unwrap().find(name).unwrap())
    }
    fn insert_symbol(&mut self, name: &'a str, symbol: Symbol) -> Result<&Symbol, Box<SemanticError>> {
        self.symbol_table.iter_mut()
            .next_back()
            .unwrap()
            .insert(name, symbol)
    }
}

#[cfg(test)]
pub mod test{
    use std::io::Cursor;
    use super::*;
    #[test]
    fn test_valid_program(){
        let program = toycc_frontend::Parser::new(
            Cursor::new("int isEven(int n){if ((n % 2) == 0) return 1; else return 0;}int main(){int a; int c; c = 44; a = c; return 0;}"),
            "test.tc",
            Some(2),
            false).parse().expect("failed to parse");
        // println!("{:#?}",program);
        let mut analyzer = SemanticAnalyzer::new();
        let c = analyzer.analyze_program(&program,  "test.tc");
        assert!(c.is_ok());

        println!("{}",c.unwrap());
    }
}