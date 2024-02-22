type Identifier = String;
type Type = String;
type VarDef = (Identifier, Vec<Identifier>, Type);

#[derive(Debug, Clone, PartialEq)]
pub enum Operator{
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Mod,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    NotEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression{
    Number(f64),
    Identifier(Identifier),
    CharLiteral(Option<char>),
    StringLiteral(String),
    FuncCall(Identifier, Vec<Expression>),
    Expr(Operator, Box<Expression>, Box<Expression>),
    Minus(Box<Expression>),
    Not(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Statement{
    ExprState(Expression),
    BreakState,
    BlockState(Vec<VarDef>, Vec<Statement>),
    IfState(Expression, Box<Statement>, Option<Box<Statement>>),
    NullState,
    ReturnState(Option<Expression>),
    WhileState(Expression, Box<Statement>),
    ReadState(Identifier,Vec<Identifier>),
    WriteState(Expression, Vec<Expression>),
    NewLineState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition{
    FuncDef(Identifier, Type, Vec<VarDef>, Statement),
    VarDef(VarDef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program(Vec<Definition>);