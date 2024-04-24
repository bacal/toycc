use crate::scanner::token::{AddOP, MulOP, RelOP, Type};
use itertools::Itertools;
use std::fmt::{Debug, Display, Formatter};

const TAB_WIDTH: usize = 2;

#[derive(Debug)]
pub struct Program {
    pub definitions: Vec<Definition>,
}

#[derive(Debug)]
pub enum Definition {
    FuncDef(FuncDef),
    VarDef(VarDef),
}

#[derive(Debug)]
pub struct FuncDef {
    identifier: String,
    toyc_type: String,
    var_def: Vec<VarDef>,
    statement: Statement,
}

#[derive(Debug)]
pub struct VarDef {
    identifiers: Vec<String>,
    toyc_type: Type,
}

impl FuncDef {
    pub fn new(
        identifier: String,
        toyc_type: String,
        var_def: Vec<VarDef>,
        statement: Statement,
    ) -> Self {
        Self {
            identifier,
            toyc_type,
            var_def,
            statement,
        }
    }
}

impl VarDef {
    pub fn new(identifiers: Vec<String>, toyc_type: Type) -> Self {
        Self {
            identifiers,
            toyc_type,
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Break,
    BlockState(Vec<VarDef>, Vec<Statement>),
    IfState(Expression, Box<Statement>, Box<Option<Statement>>),
    NullState,
    ReturnState(Option<Expression>),
    WhileState(Expression, Box<Statement>),
    ReadState(String, Option<Vec<String>>),
    WriteState(Expression, Option<Vec<Expression>>),
    NewLineState,
}

#[derive(Debug)]
pub enum Expression {
    Number(f64),
    Identifier(String),
    CharLiteral(Option<char>),
    StringLiteral(String),
    FuncCall(String, Vec<Expression>),
    Expr(Operator, Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Minus(Box<Expression>),
}

#[derive(Debug)]
pub enum Operator {
    Assign,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Or,
    And,
    LessEqual,
    LessThan,
    GreaterEqual,
    GreaterThan,
    Equal,
    NotEqual,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operator::Assign => "=",
                Operator::Plus => "+",
                Operator::Minus => "-",
                Operator::Multiply => "*",
                Operator::Divide => "/",
                Operator::Modulo => "%",
                Operator::Or => "||",
                Operator::And => "&&",
                Operator::LessEqual => "<=",
                Operator::LessThan => "<",
                Operator::GreaterEqual => ">=",
                Operator::GreaterThan => ">",
                Operator::Equal => "==",
                Operator::NotEqual => "!=",
            }
        )
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        let indent = width + TAB_WIDTH;
        for definition in &self.definitions {
            write!(
                f,
                "{:>width$}\n{:>indent$}\n{:>width$}\n",
                "Program(", definition, ")"
            )?
        }
        Ok(())
    }
}

impl Display for Definition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        let indent = width + TAB_WIDTH;

        writeln!(
            f,
            "{:>width$}",
            "Definition(",
            width = width + "Definition(".len()
        )?;

        match self {
            Definition::FuncDef(func_def) => {
                writeln!(f, "{:>indent$}", func_def)
            }
            Definition::VarDef(var_def) => {
                writeln!(f, "{:>indent$}", var_def)
            }
        }?;
        write!(f, "{:>width$}", ")", width = width + 1)
    }
}

impl Display for FuncDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        let indent = width + TAB_WIDTH;
        let dindent = indent + TAB_WIDTH;
        writeln!(
            f,
            "{:>width$}",
            "FuncDef(",
            width = width + "FuncDef(".len()
        )?;
        writeln!(
            f,
            "{:>width$}",
            "Identifier(",
            width = indent + "Identifier(".len()
        )?;
        writeln!(
            f,
            "{:>width$}",
            self.identifier,
            width = indent + TAB_WIDTH + self.identifier.len()
        )?;
        writeln!(f, "{:>dindent$}", "),")?;

        writeln!(f, "{:>width$}", "Type(", width = indent + "Type(".len())?;
        writeln!(f, "{:>width$}", self.toyc_type, width = indent + 5)?;
        writeln!(f, "{:>dindent$}", ")")?;

        for def in &self.var_def {
            writeln!(f, "{:>indent$},", def, indent = indent + 2)?;
        }

        writeln!(
            f,
            "{:>indent$}",
            "Statement(",
            indent = indent + "Statement(".len()
        )?;
        writeln!(f, "{:>dindent$}", self.statement)?;
        writeln!(f, "{:>indent$}", ")")?;

        write!(f, "{:>width$}", ")", width = width + 1)
    }
}

impl Display for VarDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        let indent = width + TAB_WIDTH;
        let dindent = indent + TAB_WIDTH;
        writeln!(f, "{:>width$}", "VarDef(", width = width + "VarDef(".len())?;
        writeln!(f, "{:>width$}", "Type(", width = indent + "Type(".len())?;
        writeln!(
            f,
            "{:>width$}",
            self.toyc_type.to_string(),
            width = dindent + 5
        )?;
        writeln!(f, "{:>indent$}", "),", indent = indent + 2)?;
        for id in &self.identifiers {
            writeln!(
                f,
                "{:>width$}",
                "Identifier(",
                width = indent + "Identifier(".len()
            )?;
            writeln!(f, "{:>width$}", id, width = dindent + id.len())?;
            writeln!(f, "{:>indent$}", "),", indent = indent + 2)?;
        }
        write!(f, "{:>width$}", ")")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        let indent = width + TAB_WIDTH;
        let dindent = indent + TAB_WIDTH;
        match self {
            Statement::Expression(e) => {
                write!(f, "{:>width$}", e, width = width + TAB_WIDTH)
            }
            Statement::Break => {
                write!(
                    f,
                    "{:>width$}",
                    "BreakStatement",
                    width = width + "BreakStatement".len()
                )
            }

            Statement::BlockState(vars, stmts) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "BlockStatement(",
                    width = width + "BlockStatement(".len()
                )?;
                writeln!(f, "{:>indent$}", "[", indent = indent + 1)?;
                let u_formatted = vars.iter().map(|i| format!("{:>dindent$}", i)).join(",\n");
                writeln!(f, "{}", u_formatted)?;
                writeln!(f, "{:>indent$}", "],", indent = indent + 2)?;

                writeln!(f, "{:>indent$}", "[", indent = indent + 1)?;
                let v_formatted = stmts.iter().map(|i| format!("{:>dindent$}", i)).join(",\n");
                writeln!(f, "{:>width$}", v_formatted)?;
                write!(f, "{:>width$}", "],", width = width + 2)
            }

            Statement::IfState(expr, if_stmt, else_stmt) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "IfStatement(",
                    width = width + "IfStatement(".len()
                )?;
                writeln!(f, "{:>indent$}", "[", indent = indent + 1)?;
                writeln!(f, "{:>dindent$},", expr)?;
                writeln!(f, "{:>indent$},", if_stmt.as_ref())?;
                if let Some(expr) = else_stmt.as_ref() {
                    writeln!(f, "{:>indent$}", expr)?;
                };
                write!(f, "{:>indent$}", "]", indent = indent + 1)
            }

            Statement::NullState => {
                write!(
                    f,
                    "{:>width$}",
                    "NullStatement",
                    width = width + "NullStatement".len()
                )
            }

            Statement::ReturnState(e) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "ReturnStatement(",
                    width = width + "ReturnStatement(".len()
                )?;
                if let Some(e) = e {
                    writeln!(f, "{:>indent$}", e)?;
                }
                write!(f, "{:>width$}", ")", width = width + 1)
            }

            Statement::WhileState(expr, stmt) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "WhileStatement(",
                    width = width + "WhileStatement(".len()
                )?;
                writeln!(f, "{:>width$}", "[", width = width + 1)?;
                writeln!(f, "{:>indent$}", expr, indent = indent)?;
                writeln!(f, "{:>width$}", "],", width = width + 2)?;
                writeln!(f, "{:>width$}", "[", width = width + 1)?;
                writeln!(f, "{:>indent$}", stmt.as_ref())?;
                writeln!(f, "{:>width$}", "]", width = width + 1)?;
                write!(f, "{:>width$}", ")", width = width + 1)
            }

            Statement::ReadState(id, s1) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "ReadStatement(",
                    width = width + "ReadStatement(".len()
                )?;
                writeln!(f, "{:>indent$},", id, indent = indent + id.len())?;
                let s1_formatted = match s1 {
                    Some(s1) => s1
                        .iter()
                        .map(|f| format!("{:>dindent$}", f, dindent = dindent + f.len()))
                        .join(",\n"),
                    None => format!("{:>dindent$}", "[]"),
                };
                writeln!(f, "{:>indent$}", s1_formatted)?;
                write!(f, "{:>width$}", ")", width = width + 1)
            }

            Statement::WriteState(e, others) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "WriteStatement(",
                    width = width + "WriteStatement(".len()
                )?;
                writeln!(f, "{:>indent$},", e)?;
                if let Some(others) = others {
                    for expr in others {
                        writeln!(f, "{:>indent$},", expr)?;
                    }
                }
                write!(f, "{:>width$}", ")", width = width + 1)
            }

            Statement::NewLineState => {
                write!(
                    f,
                    "{:>width$}",
                    "NewLineStatement",
                    width = width + "NewLineStatement".len()
                )
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        writeln!(
            f,
            "{:>width$}",
            "Expression(",
            width = width + "Expression(".len()
        )?;
        let width = width + TAB_WIDTH;
        let indent = width + TAB_WIDTH;
        match self {
            Expression::Number(num) => {
                writeln!(f, "{:>width$}", "Number(", width = width + "Number(".len())?;
                writeln!(
                    f,
                    "{:>indent$}",
                    num,
                    indent = indent + num.to_string().len()
                )?;
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }
            Expression::Identifier(id) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "Identifier(",
                    width = width + "Identifier(".len()
                )?;
                writeln!(f, "{:>indent$}", id, indent = indent + id.len())?;
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }
            Expression::CharLiteral(cl) => {
                let cl = match cl {
                    Some(cl) => format!("{cl}"),
                    None => "".to_string(),
                };
                writeln!(
                    f,
                    "{:>width$}",
                    "CharLiteral(",
                    width = width + "CharLiteral(".len()
                )?;
                writeln!(f, "{:>indent$}", cl, indent = indent + cl.len())?;
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }

            Expression::StringLiteral(s) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "StringLiteral(",
                    width = width + "StringLiteral(".len()
                )?;
                writeln!(f, "{:>indent$}", s, indent = indent + s.len())?;
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }
            Expression::FuncCall(name, expressions) => {
                writeln!(
                    f,
                    "{:>width$}",
                    "FunctionCall(",
                    width = width + "FunctionCall(".len()
                )?;
                writeln!(f, "{:>indent$},", name, indent = indent + name.len())?;
                for expr in expressions {
                    writeln!(f, "{:>indent$},", expr, indent = indent + 1)?;
                }
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }
            Expression::Expr(op, expra, exprb) => {
                writeln!(f, "{:>width$}", "Expr(", width = width + "Expr(".len())?;
                writeln!(
                    f,
                    "{:>indent$},",
                    op.to_string(),
                    indent = indent + op.to_string().len()
                )?;
                writeln!(f, "{:>indent$},", expra, indent = indent + 1)?;
                writeln!(f, "{:>indent$},", exprb, indent = indent + 1)?;
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }
            Expression::Not(expr) => {
                writeln!(f, "{:>width$}", "Not(", width = width + "Not(".len())?;
                writeln!(f, "{:>indent$},", expr, indent = indent + 1)?;
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }
            Expression::Minus(expr) => {
                writeln!(f, "{:>width$}", "Minus(")?;
                writeln!(f, "{:>indent$},", expr, indent = indent + 1)?;
                writeln!(f, "{:>width$}", ")", width = width + 1)
            }
        }?;
        write!(f, "{:>width$}", ')', width = width + 1)
    }
}

impl From<AddOP> for Operator {
    fn from(value: AddOP) -> Self {
        match value {
            AddOP::Plus => Self::Plus,
            AddOP::Minus => Self::Minus,
            AddOP::Or => Self::Or,
        }
    }
}
impl From<&AddOP> for Operator {
    fn from(value: &AddOP) -> Self {
        value.into()
    }
}

impl From<&RelOP> for Operator {
    fn from(value: &RelOP) -> Self {
        value.into()
    }
}

impl From<&MulOP> for Operator {
    fn from(value: &MulOP) -> Self {
        value.into()
    }
}

impl From<MulOP> for Operator {
    fn from(value: MulOP) -> Self {
        match value {
            MulOP::Multiply => Self::Multiply,
            MulOP::Divide => Self::Divide,
            MulOP::Mod => Self::Modulo,
            MulOP::And => Self::And,
        }
    }
}

impl From<RelOP> for Operator {
    fn from(value: RelOP) -> Self {
        match value {
            RelOP::EqualsEquals => Self::Equal,
            RelOP::NotEquals => Self::NotEqual,
            RelOP::LessThan => Self::LessThan,
            RelOP::LessEqual => Self::LessEqual,
            RelOP::GreaterEqual => Self::GreaterEqual,
            RelOP::GreaterThan => Self::GreaterThan,
        }
    }
}
