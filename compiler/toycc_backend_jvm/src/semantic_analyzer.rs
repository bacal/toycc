use crate::error::{SemanticError, SemanticErrorKind};
use crate::symbol_table::{Function, Symbol, SymbolTable};
use itertools::Itertools;
use toycc_frontend::ast::{Definition, Expression, FuncDef, Operator, Program, Statement, VarDef};
use toycc_frontend::Type;

const CLASS_INIT_HEADER: &str = r#"
.method <init>()V
    aload_0
    invokespecial java/lang/Object/<init>()V
    return
.end method

"#;

#[derive(Default)]
pub struct SemanticAnalyzer<'a> {
    class_name: &'a str,
    symbol_table: Vec<SymbolTable<'a>>,
    conditional_count: usize,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(class_name: &'a str) -> Self {
        Self {
            conditional_count: 0,
            class_name,
            symbol_table: vec![SymbolTable::default(); 1],
        }
    }
    pub fn analyze_program(
        &mut self,
        program: &'a Program,
    ) -> Result<String, Box<SemanticError>> {
        let mut jasmin_program = format!(
            ".class public {}\n.super java/lang/Object{}\n",
            self.class_name,
            CLASS_INIT_HEADER
        );
        let x: Vec<_> = program
            .definitions
            .iter()
            .map(|def| self.analyze_definition(def))
            .fold_ok(vec![], |mut acc, mut e| {
                acc.append(&mut e);
                acc
            })?;

        jasmin_program += x.join("\t\n").as_str();

        Ok(jasmin_program)
    }

    fn analyze_definition(
        &mut self,
        definition: &'a Definition,
    ) -> Result<Vec<String>, Box<SemanticError>> {
        match definition {
            Definition::FuncDef(func_def) => self.analyze_func_def(func_def),
            Definition::VarDef(var_def) => self.analyze_var_def(var_def),
        }
    }

    fn analyze_func_def(
        &mut self,
        func_def: &'a FuncDef,
    ) -> Result<Vec<String>, Box<SemanticError>> {
        let mut instructions = vec![];
        let mut return_type = match func_def.toyc_type {
            Type::Int => "I",
            Type::Char => "C",
        };
        self.push_scope();

        for var_def in &func_def.var_def {
            self.analyze_var_def(var_def)?;
        }

        let mut args: Vec<_> = func_def
            .var_def
            .iter()
            .map(|arg| match arg.toyc_type {
                Type::Int => "I".to_string(),
                Type::Char => "C".to_string(),
            })
            .collect();
        if func_def.identifier == "main" {
            args = vec!["[Ljava/lang/String;".to_owned()];
            return_type = "V";
        }
        let mut body = self.analyze_statement(&func_def.statement)?;

        self.pop_scope();
        let function = Function::new(
            func_def.identifier.clone(),
            args.clone(),
            body.clone(),
            func_def.toyc_type.clone(),
        );

        self.insert_symbol(func_def.identifier.as_str(), Symbol::Function(function))?;

        body.iter_mut()
            .filter(|f| !f.starts_with('.') && !f.ends_with(':'))
            .for_each(|f| f.insert(0, '\t'));

        instructions.push(format!(".method public static {}({}){}\n\t.limit stack 1000\n\t.limit locals 1000\n{}\n.end method\n",
                              func_def.identifier,
                              args.join(""),
                              return_type,
                              body.join("\n")));
        Ok(instructions)
    }
    fn analyze_var_def(&mut self, var_def: &'a VarDef) -> Result<Vec<String>, Box<SemanticError>> {
        for id in &var_def.identifiers {
            let pos = self.symbol_table.iter_mut().next_back().unwrap().len();
            self.insert_symbol(
                id.as_str(),
                Symbol::Variable(var_def.toyc_type.clone(), pos),
            )?;
        }

        Ok(vec![])
    }

    fn push_scope(&mut self) {
        self.symbol_table
            .push(self.symbol_table.iter().next_back().unwrap().clone())
    }

    fn analyze_statement(
        &mut self,
        statement: &'a Statement,
    ) -> Result<Vec<String>, Box<SemanticError>> {
        let mut instructions = vec![];
        match statement {
            Statement::Expression(expr) => instructions.append(&mut self.analyze_expression(expr)?),
            Statement::Break => instructions.push(format!("goto CE{}", self.conditional_count - 1)),
            Statement::BlockState(var_defs, statements) => {
                instructions.append(
                    &mut var_defs
                        .iter()
                        .map(|a| self.analyze_var_def(a))
                        .collect::<Result<Vec<_>, Box<SemanticError>>>()?
                        .iter()
                        .flatten()
                        .map(Clone::clone)
                        .collect::<Vec<_>>(),
                );
                instructions.append(
                    &mut statements
                        .iter()
                        .map(|s| self.analyze_statement(s))
                        .collect::<Result<Vec<_>, Box<SemanticError>>>()?
                        .iter()
                        .flatten()
                        .map(Clone::clone)
                        .collect::<Vec<_>>(),
                );
            }
            Statement::IfState(expr, statement, else_stmt) => {
                self.conditional_count += 1;
                let then_label = format!("CT{}", self.conditional_count);
                let else_label = format!("CL{}", self.conditional_count);
                let end_label = format!("CE{}", self.conditional_count);
                match expr {
                    Expression::Expr(..) => {
                        instructions.append(&mut self.analyze_expression(expr)?);
                        match else_stmt.as_ref() {
                            Some(_) => instructions.push(format!("goto {else_label}")),
                            None => instructions.push(format!("goto {end_label}")),
                        }
                        instructions.push(format!("{then_label}:"))
                    }
                    _ => {
                        instructions.append(&mut self.analyze_expression(expr)?);
                        instructions.push("iconst_1".to_string());
                        instructions.push(format!("if_icmpeq {then_label}"));
                        match else_stmt.as_ref() {
                            Some(_) => instructions.push(format!("goto {else_label}")),
                            None => instructions.push(format!("goto {end_label}")),
                        }
                        instructions.push(format!("{then_label}:"));
                    }
                };
                instructions.append(&mut self.analyze_statement(statement)?);

                if let Some(else_statement) = else_stmt.as_ref() {
                    instructions.push(format!("goto {end_label}"));
                    instructions.push(format!("{else_label}:"));
                    instructions.append(&mut self.analyze_statement(else_statement)?);
                }
                instructions.push(format!("{end_label}:"));
            }
            Statement::NullState => instructions.push("nop".to_string()),
            Statement::ReturnState(arg) => match arg {
                Some(arg) => {
                    instructions.append(&mut self.analyze_expression(arg)?);
                    instructions.push("ireturn".to_string());
                }
                None => instructions.push("return".to_string()),
            },
            Statement::WhileState(expr, statement) => {
                self.conditional_count += 1;
                let then_label = format!("CT{}", self.conditional_count);
                let end_label = format!("CE{}", self.conditional_count);

                instructions.append(&mut self.analyze_expression(expr)?);

                match expr {
                    Expression::Expr(..) => {
                        instructions.push(format!("goto {end_label}"));
                    }
                    _ => {
                        instructions.push("iconst_1".to_string());
                        instructions.push(format!("if_icmpne {end_label}"));
                    }
                }
                instructions.push(format!("{then_label}:"));
                instructions.append(&mut self.analyze_statement(statement)?);

                match expr {
                    Expression::Expr(..) => {
                        instructions.append(&mut self.analyze_expression(expr)?)
                    }
                    _ => {
                        instructions.append(&mut self.analyze_expression(expr)?);
                        instructions.push("iconst_1".to_string());
                        instructions.push(format!("if_icmpeq {then_label}"));
                    }
                }
                instructions.push(format!("{end_label}:"));
            }
            Statement::ReadState(name, others) => {
                if self
                    .insert_symbol("JAVA_SCANNER", Symbol::Variable(Type::Int, 900))
                    .is_ok()
                {
                    instructions.push("new java/util/Scanner".to_owned());
                    instructions.push("dup".to_owned());
                    instructions
                        .push("getstatic java/lang/System/in Ljava/io/InputStream;".to_owned());
                    instructions.push(
                        "invokespecial java/util/Scanner/<init>(Ljava/io/InputStream;)V".to_owned(),
                    );
                    instructions.push("astore 900".to_owned());
                }
                instructions.push("aload 900".to_owned());

                match self.get_symbol(name)? {
                    Symbol::Variable(toyc_type, num) => {
                        match toyc_type {
                            Type::Int => instructions
                                .push("invokevirtual java/util/Scanner/nextInt()I".to_string()),
                            Type::Char => instructions
                                .push("invokevirtual java/util/Scanner/nextChar()C".to_string()),
                        }
                        instructions.push(format!("istore {num}"))
                    }
                    _ => {
                        return Err(Box::new(SemanticError::new(
                            SemanticErrorKind::ExpectedIdentifier,
                        )))
                    }
                }

                if let Some(others) = others {
                    for name in others {
                        instructions.push("aload 900".to_owned());
                        match self.get_symbol(name)? {
                            Symbol::Variable(toyc_type, num) => {
                                match toyc_type {
                                    Type::Int => instructions.push(
                                        "invokevirtual java/util/Scanner/nextInt()I".to_string(),
                                    ),
                                    Type::Char => instructions.push(
                                        "invokevirtual java/util/Scanner/nextChar()C".to_string(),
                                    ),
                                }
                                instructions.push(format!("istore {num}"))
                            }
                            _ => {
                                return Err(Box::new(SemanticError::new(
                                    SemanticErrorKind::ExpectedIdentifier,
                                )))
                            }
                        }
                    }
                }
            }
            Statement::WriteState(expr, others) => {
                let arg_type = self.get_return_type(expr)?;
                match arg_type {
                    "S" => {
                        instructions.append(&mut self.analyze_expression(expr)?);
                        instructions.push("astore 901".to_string());
                        instructions.push(
                            "getstatic java/lang/System/out Ljava/io/PrintStream;".to_string(),
                        );
                        instructions.push("aload 901".to_string());
                        instructions.push(
                            "invokevirtual java/io/PrintStream/print(Ljava/lang/String;)V"
                                .to_string(),
                        )
                    }
                    format => {
                        instructions.push(
                            "getstatic java/lang/System/out Ljava/io/PrintStream;".to_string(),
                        );
                        instructions.append(&mut self.analyze_expression(expr)?);
                        instructions.push(format!(
                            "invokevirtual java/io/PrintStream/print({format})V"
                        ))
                    }
                }
                if let Some(others) = others {
                    for expr in others {
                        let arg_type = self.get_return_type(expr)?;
                        match arg_type {
                            "S" => {
                                instructions.append(&mut self.analyze_expression(expr)?);
                                instructions.push("astore 901".to_string());
                                instructions.push(
                                    "getstatic java/lang/System/out Ljava/io/PrintStream;"
                                        .to_string(),
                                );
                                instructions.push("aload 901".to_string());
                                instructions.push(
                                    "invokevirtual java/io/PrintStream/print(Ljava/lang/String;)V"
                                        .to_string(),
                                )
                            }
                            format => {
                                instructions.push(
                                    "getstatic java/lang/System/out Ljava/io/PrintStream;"
                                        .to_string(),
                                );
                                instructions.append(&mut self.analyze_expression(expr)?);
                                instructions.push(format!(
                                    "invokevirtual java/io/PrintStream/print({format})V"
                                ))
                            }
                        }
                    }
                }
            }
            Statement::NewLineState => {
                instructions
                    .push("getstatic java/lang/System/out Ljava/io/PrintStream;".to_string());
                instructions.push("invokevirtual java/io/PrintStream/println()V".to_string());
            }
        }
        Ok(instructions)
    }

    fn analyze_expression(
        &mut self,
        expression: &'a Expression,
    ) -> Result<Vec<String>, Box<SemanticError>> {
        let mut instructions = vec![];
        match expression {
            Expression::Number(num) => {
                if *num <= 5.0 {
                    instructions.push(format!("iconst_{num}"));
                } else {
                    instructions.push(format!("bipush {num}"));
                }
            }
            Expression::Identifier(id) => match self.get_symbol(id)? {
                Symbol::Variable(_, num) => instructions.push(format!("iload {num}")),
                _ => {
                    return Err(Box::new(SemanticError::new(
                        SemanticErrorKind::ExpectedIdentifier,
                    )))
                }
            },
            Expression::CharLiteral(c) => {
                if let Some(c) = c {
                    instructions.push(format!("bipush {}", *c as u32));
                }
            }
            Expression::StringLiteral(s) => {
                instructions.push(format!("ldc \"{s}\""));
            }

            Expression::FuncCall(name, arguments) => {
                let program_name = self.class_name;
                instructions.append(
                    &mut arguments
                        .iter()
                        .map(|a| self.analyze_expression(a))
                        .collect::<Result<Vec<_>, Box<SemanticError>>>()?
                        .iter()
                        .flatten()
                        .map(Clone::clone)
                        .collect::<Vec<_>>(),
                );

                if let Symbol::Function(func) = self.get_symbol(name)? {
                    let call = format!(
                        "invokestatic {}/{name}({}){}",
                        program_name,
                        func.arguments.clone().join(""),
                        match func.return_type {
                            Type::Int => "I",
                            Type::Char => "C",
                        }
                    );
                    instructions.push(call);
                } else {
                    return Err(Box::new(SemanticError::new(
                        SemanticErrorKind::UndeclaredFunction(name.clone()),
                    )));
                }
            }
            
            Expression::Expr(op, expra, exprb) => {
                let then_label = format!("CT{}", self.conditional_count);

                if *op != Operator::Assign {
                    instructions.append(&mut self.analyze_expression(expra)?);
                }
                instructions.append(&mut self.analyze_expression(exprb)?);
                match op {
                    Operator::Plus => instructions.push("iadd".to_owned()),
                    Operator::Minus => instructions.push("isub".to_owned()),
                    Operator::Multiply => instructions.push("imul".to_owned()),
                    Operator::Divide => {
                        if let Expression::Number(num) = exprb.as_ref() {
                            if *num == 0.0 {
                                return Err(Box::new(SemanticError::new(
                                    SemanticErrorKind::DivisionByZero,
                                )));
                            }
                        }
                        instructions.push("idiv".to_owned())
                    }
                    Operator::Modulo => {
                        if let Expression::Number(num) = exprb.as_ref() {
                            if *num == 0.0 {
                                return Err(Box::new(SemanticError::new(
                                    SemanticErrorKind::DivisionByZero,
                                )));
                            }
                        }
                        instructions.push("irem".to_owned())
                    }
                    Operator::Or => instructions.push("ior".to_owned()),
                    Operator::And => instructions.push("iand".to_owned()),
                    Operator::LessEqual => instructions.push(format!("if_icmple {then_label}")),
                    Operator::LessThan => instructions.push(format!("if_icmplt {then_label}")),
                    Operator::GreaterEqual => instructions.push(format!("if_icmpge {then_label}")),
                    Operator::GreaterThan => instructions.push(format!("if_icmpgt {then_label}")),
                    Operator::Equal => instructions.push(format!("if_icmpeq {then_label}")),
                    Operator::NotEqual => instructions.push(format!("if_icmpne {then_label}")),
                    Operator::Assign => match expra.as_ref() {
                        Expression::Identifier(id) => match self.get_symbol(id)? {
                            Symbol::Variable(_, num) => {
                                instructions.push(format!("istore {num}"));
                            }
                            _ => {
                                return Err(Box::new(SemanticError::new(
                                    SemanticErrorKind::UndeclaredIdentifier(id.clone()),
                                )))
                            }
                        },
                        _ => {
                            return Err(Box::new(SemanticError::new(
                                SemanticErrorKind::ExpectedIdentifier,
                            )))
                        }
                    },
                };
            }
            Expression::Not(expr) => {
                instructions.append(&mut self.analyze_expression(expr)?);
                instructions.push("iconst_m1".to_owned());
                instructions.push("ixor".to_owned());
            }
            Expression::Minus(expr) => {
                instructions.append(&mut self.analyze_expression(expr)?);
                instructions.push("inot".to_owned());
            }
        }

        Ok(instructions)
    }

    fn pop_scope(&mut self) {
        self.symbol_table.pop();
    }

    fn get_symbol(&mut self, name: &'a str) -> Result<&Symbol, Box<SemanticError>> {
        self.symbol_table
            .iter_mut()
            .next_back()
            .unwrap()
            .find(name)
            .ok_or(Box::new(SemanticError::new(
                SemanticErrorKind::UndeclaredIdentifier(name.to_string()),
            )))
    }
    fn insert_symbol(
        &mut self,
        name: &'a str,
        symbol: Symbol,
    ) -> Result<&Symbol, Box<SemanticError>> {
        self.symbol_table
            .iter_mut()
            .next_back()
            .unwrap()
            .insert(name, symbol)
    }

    fn get_return_type(
        &mut self,
        expr: &'a Expression,
    ) -> Result<&'static str, Box<SemanticError>> {
        Ok(match expr {
            Expression::Number(_) => "I",
            Expression::Identifier(id) => match self.get_symbol(id)? {
                Symbol::Variable(t_type, _) => match t_type {
                    Type::Int => "I",
                    Type::Char => "C",
                },
                Symbol::Function(f) => match f.return_type {
                    Type::Int => "I",
                    Type::Char => "C",
                },
            },
            Expression::CharLiteral(_) => "C",
            Expression::StringLiteral(_) => "S",
            Expression::FuncCall(name, _) => match self.get_symbol(name)? {
                Symbol::Function(f) => match f.return_type {
                    Type::Int => "I",
                    Type::Char => "C",
                },
                _ => {
                    return Err(Box::new(SemanticError::new(
                        SemanticErrorKind::ExpectedFunction,
                    )))
                }
            },
            Expression::Expr(_, a, _) => self.get_return_type(a)?,
            Expression::Not(val) => self.get_return_type(val)?,
            Expression::Minus(val) => self.get_return_type(val)?,
        })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn test_valid_program() {
        let program = toycc_frontend::Parser::new(
            Cursor::new("int isEven(int n){if ((n % 2) == 0) return 1; else return 0;}int main(){int a; int c; c = 44; a = c; return 0;}"),
            "test.tc",
            Some(2),
            false).parse().expect("failed to parse");
        // println!("{:#?}",program);
        let mut analyzer = SemanticAnalyzer::new();
        let c = analyzer.analyze_program(&program, "test.tc");
        assert!(c.is_ok());

        println!("{}", c.unwrap());
    }
}
