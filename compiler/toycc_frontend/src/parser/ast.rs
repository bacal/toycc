use crate::scanner::token::{AddOP, MulOP, RelOP, Type};

#[derive(Debug)]
pub enum Program {
    Definition(Vec<Definition>),
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
