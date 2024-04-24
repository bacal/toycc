use crate::scanner::token::{AddOP, MulOP, RelOP, Type};
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
    pub identifier: String,
    pub toyc_type: Type,
    pub var_def: Vec<VarDef>,
    pub statement: Statement,
}

#[derive(Debug)]
pub struct VarDef {
    pub identifiers: Vec<String>,
    pub toyc_type: Type,
}

impl FuncDef {
    pub fn new(
        identifier: String,
        toyc_type: Type,
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

#[derive(Debug, PartialEq)]
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

// impl Display for Program {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let width = f.width().unwrap_or_default();
//         for definition in &self.definitions {
//             write!(f, "{:>width$}\n{:>width$}\n{:>width$}", "Program(", definition, ")", width = width + TAB_WIDTH)?
//         }
//         Ok(())
//     }
// }
//
// impl Display for Definition {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let width = f.width().unwrap_or_default();
//         let variant = match self {
//             Definition::FuncDef(func_def) => format!("{:>width$}", func_def, width = width + TAB_WIDTH),
//             Definition::VarDef(var_def) => format!("{:>width$}", var_def, width = width + TAB_WIDTH),
//         };
//         write!(f, "{:>width$}\n{:>width$}\n", "Definition(", variant)?;
//         write!(f, "{:>width$}", ")", width = width - 1)
//     }
// }
//
// impl Display for FuncDef {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "FuncDef(\n{:4}\n)", self)
//     }
// }
//
// impl Display for VarDef {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "VarDef(\n{:4}\n)", self)
//     }
// }
//
// impl Display for Statement {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         todo!();
//     }
// }

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
