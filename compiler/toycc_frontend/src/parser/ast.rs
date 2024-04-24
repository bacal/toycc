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

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        for definition in &self.definitions {
            write!(
                f,
                "{:>width$}\n{:>width$}\n{:>width$}",
                "Program(",
                definition,
                ")",
                width = width + TAB_WIDTH
            )?
        }
        Ok(())
    }
}

impl Display for Definition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        let variant = match self {
            Definition::FuncDef(func_def) => {
                format!("{:>width$}", func_def, width = width + TAB_WIDTH)
            }
            Definition::VarDef(var_def) => {
                format!("{:>width$}", var_def, width = width + TAB_WIDTH)
            }
        };
        write!(f, "{:>width$}\n{:>width$}\n", "Definition(", variant)?;
        write!(f, "{:>width$}", ")", width = width - 1)
    }
}

impl Display for FuncDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FuncDef(\n{:4}\n)", self)
    }
}

impl Display for VarDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "VarDef(\n{:4}\n)", self)
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        let variant = match self {
            Statement::Expression(e) => {
                write!(f, "{:>width$}", e, width = width + TAB_WIDTH);
            }

            Statement::Break => {
                write!(
                    f,
                    "{:>width$}",
                    "BreakStatement\n",
                    width = width + TAB_WIDTH
                );
            }

            Statement::BlockState(u, v) => {
                write!(
                    f,
                    "{:>width$}",
                    "BlockStatement(\n",
                    width = width + TAB_WIDTH
                );
                write!(f, "{:>width$}", "[\n", width = width + TAB_WIDTH);
                let u_formatted = u
                    .iter()
                    .map(|i| format!("{:>width$}", i, width = width + 2 * TAB_WIDTH))
                    .join(",\n");
                write!(f, "{:>width$}", u_formatted);
                write!(f, "{:>width$}", "],\n", width = width + TAB_WIDTH);
                let v_formatted = v
                    .iter()
                    .map(|i| format!("{:>width$}", i, width = width + 2 * TAB_WIDTH))
                    .join(",\n");
                write!(f, "{:>width$}", v_formatted);
                write!(f, "{:>width$}", "],\n", width = width + TAB_WIDTH);
            }

            Statement::IfState(e, s, s1) => {
                write!(f, "{:>width$}", "IfStatement(\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", "[\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", s.as_ref(), width = width + 2 * TAB_WIDTH);
                if let Some(expr) = s1.as_ref() {
                    write!(f, "{:>width$}", expr, width = width + 2 * TAB_WIDTH);
                };
                write!(f, "{:>width$}", "]\n", width = width + TAB_WIDTH);
            }

            Statement::NullState => {
                write!(f, "{:>width$}", "NullStatement", width = width + TAB_WIDTH);
            }

            Statement::ReturnState(e) => {
                write!(
                    f,
                    "{:>width$}",
                    "ReturnStatement(",
                    width = width + TAB_WIDTH
                );
                write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH);
            }

            Statement::WhileState(e, s) => {
                write!(
                    f,
                    "{:>width$}",
                    "WhileStatement(",
                    width = width + TAB_WIDTH
                );
                write!(f, "{:>width$}", "[\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", "],\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", "[\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", s.as_ref(), width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", "]\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH);
            }

            Statement::ReadState(s, s1) => {
                write!(f, "{:>width$}", "ReadStatement(", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", s, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", ",\n", width = width + 2 * TAB_WIDTH);
                let s1_formatted = match s1 {
                    Some(s1) => s1
                        .iter()
                        .map(|f| format!("{:>width$}", f, width = width + 2 * TAB_WIDTH))
                        .join(",\n"),
                    None => format!("{:>width$}", "[]"),
                };
                write!(f, "{:>width$}", s1_formatted);
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH);
            }

            Statement::WriteState(e, e1) => {
                write!(
                    f,
                    "{:>width$}",
                    "WriteStatement(",
                    width = width + TAB_WIDTH
                );
                write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", ",\n", width = width + 2 * TAB_WIDTH);
                let e1_formatted = e1
                    .unwrap()
                    .iter()
                    .map(|f| format!("{:>width$}", f, width = width + 2 * TAB_WIDTH))
                    .join(",\n");

                write!(f, "{:>width$}", e1_formatted);
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH);
            }

            Statement::NewLineState => {
                write!(
                    f,
                    "{:>width$}",
                    "NewLineStatement",
                    width = width + TAB_WIDTH
                );
            }
        };
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
