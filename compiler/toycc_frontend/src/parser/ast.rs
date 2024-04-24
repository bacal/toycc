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

impl Display for Operator{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",match self{
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
        })
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        for definition in &self.definitions {
            write!(
                f,
                "{:>width$}\n{:>width2$}\n{:>width$}\n",
                "Program(",
                definition,
                ")",
                width = width + TAB_WIDTH,
                width2 = width + (2*TAB_WIDTH),
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
                format!("{:>width$}", func_def, width = width + TAB_WIDTH +8)
            }
            Definition::VarDef(var_def) => {
                format!("{:>width$}", var_def, width = width + TAB_WIDTH+ 7)
            }
        };
        write!(f, "{:>width$}\n{:>width$}\n", "Definition(", variant, width = width + TAB_WIDTH + 11)?;
        write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH)
    }
}

impl Display for FuncDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        write!(f, "{:>width$}\n","FuncDef(", width = width + TAB_WIDTH);
        write!(f, "{:>width$}\n","Identifier(", width = width + 11 - 2*TAB_WIDTH );
        write!(f,"{:>width$}\n", self.identifier, width = width + 11 - 4*TAB_WIDTH );
        write!(f, "{:>width$}\n","),");

        write!(f, "{:>width$}\n","Type(", width = width + 2*TAB_WIDTH + 8);
        write!(f,"{:>width$}\n", self.toyc_type, width = width + 4*TAB_WIDTH + 6);
        write!(f, "{:>width$}\n","),", width = width + 2*TAB_WIDTH);

        for def in &self.var_def{
            write!(f, "{:>width$}\n","VarDef(", width = width + 2*TAB_WIDTH);
            write!(f,"{:>width$}\n", def, width = width + 4*TAB_WIDTH + 7);
            write!(f, "{:>width$}\n","),", width = width + 2*TAB_WIDTH );
        }

        write!(f, "{:>width$}\n","Statement(", width = width + 2*TAB_WIDTH);
        write!(f,"{:>width$}\n", self.statement, width = width + 4*TAB_WIDTH);
        write!(f, "{:>width$}\n","),", width = width + TAB_WIDTH + 7);

        write!(f, "{:>width$}\n",")\n", width = width + TAB_WIDTH)

    }
}

impl Display for VarDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        write!(f, "{:>width$}\n","VarDef(");
        write!(f, "{:>width$}\n","Type(", width = width + TAB_WIDTH);
        write!(f,"{:>width$}\n", self.toyc_type, width = width + 2*TAB_WIDTH);
        write!(f, "{:>width$}\n","),", width = width + TAB_WIDTH);
        for def in &self.identifiers{
            write!(f, "{:>width$}\n","VarDef(", width = width + TAB_WIDTH);
            write!(f,"{:>width$}\n", def, width = width + 2*TAB_WIDTH);
            write!(f, "{:>width$}\n","),", width = width + TAB_WIDTH);
        }
        write!(f, "{:>width$}\n",")\n")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        match self {
            Statement::Expression(e) => {
                write!(f, "{:>width$}", e, width = width + TAB_WIDTH)
            }

            Statement::Break => {
                write!(
                    f,
                    "{:>width$}",
                    "BreakStatement\n",
                    width = width + TAB_WIDTH
                )
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
                write!(f, "{:>width$}", "],\n", width = width + TAB_WIDTH)
            }

            Statement::IfState(e, s, s1) => {
                write!(f, "{:>width$}", "IfStatement(\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", "[\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", s.as_ref(), width = width + 2 * TAB_WIDTH);
                if let Some(expr) = s1.as_ref() {
                    write!(f, "{:>width$}", expr, width = width + 2 * TAB_WIDTH);
                };
                write!(f, "{:>width$}", "]\n", width = width + TAB_WIDTH)
            }

            Statement::NullState => {
                write!(f, "{:>width$}", "NullStatement\n", width = width + TAB_WIDTH)
            }

            Statement::ReturnState(e) => {
                write!(
                    f,
                    "{:>width$}",
                    "ReturnStatement(\n",
                    width = width + TAB_WIDTH
                );
                if let Some(e) = e{
                    write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                }
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH)
            }

            Statement::WhileState(e, s) => {
                write!(
                    f,
                    "{:>width$}",
                    "WhileStatement(\n",
                    width = width + TAB_WIDTH
                );
                write!(f, "{:>width$}", "[\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", "],\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", "[\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", s.as_ref(), width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", "]\n", width = width + TAB_WIDTH);
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH)
            }

            Statement::ReadState(s, s1) => {
                write!(f, "{:>width$}", "ReadStatement(\n", width = width + TAB_WIDTH);
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
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH)
            }

            Statement::WriteState(e, e1) => {
                write!(
                    f,
                    "{:>width$}",
                    "WriteStatement(\n",
                    width = width + TAB_WIDTH
                );
                write!(f, "{:>width$}", e, width = width + 2 * TAB_WIDTH);
                write!(f, "{:>width$}", ",\n", width = width + 2 * TAB_WIDTH);
                if let Some(e1) = e1{
                    for expr in e1{
                        write!(f, "{:>width$}", expr, width = width + 2 * TAB_WIDTH);
                    }
                }
                write!(f, "{:>width$}", ")\n", width = width + TAB_WIDTH)
            }

            Statement::NewLineState => {
                write!(
                    f,
                    "{:>width$}",
                    "NewLineStatement\n",
                    width = width + TAB_WIDTH
                )
            }
        }
    }
}

impl Display for Expression{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or_default();
        write!(f, "{:>width$}\n","Expression(");
        match self{
            Expression::Number(num) => {
                write!(f, "{:>width$}\n","Identifier(");
                write!(f,"{:>width$}\n", num, width = width + 2*TAB_WIDTH);
                write!(f, "{:>width$}\n",")")
            },
            Expression::Identifier(id) => {
                write!(f, "{:>width$}\n","Identifier(");
                write!(f,"{:>width$}\n", id, width = width + 2*TAB_WIDTH);
                write!(f, "{:>width$}\n",")")
            },
            Expression::CharLiteral(cl) => {
                let cl =  match cl{
                    Some(cl) => format!("{cl}"),
                    None => "".to_string(),
                };
                write!(f, "{:>width$}\n","CharLiteral(");
                write!(f,"{:>width$}\n", cl, width = width + 2*TAB_WIDTH);
                write!(f, "{:>width$}\n",")")
            },

            Expression::StringLiteral(s) => {
                write!(f, "{:>width$}\n","StringLiteral(");
                write!(f,"{:>width$}\n", s, width = width + 2*TAB_WIDTH);
                write!(f, "{:>width$}\n",")")
            },
            Expression::FuncCall(name, expressions) => {
                write!(f, "{:>width$}\n","FuncionCall(");
                write!(f,"{:>width$},\n", name, width = width + 2*TAB_WIDTH);
                for expr in expressions{
                    write!(f,"{:>width$},\n", expr, width = width + 2*TAB_WIDTH);
                }
                write!(f, "{:>width$}\n",")")
            },
            Expression::Expr(op, expra, exprb) => {
                write!(f, "{:>width$}\n","Expr(");
                write!(f,"{:>width$},\n", op, width = width + 2*TAB_WIDTH);
                write!(f,"{:>width$},\n", expra, width = width + 2*TAB_WIDTH);
                write!(f,"{:>width$},\n", exprb, width = width + 2*TAB_WIDTH);
                write!(f, "{:>width$}\n",")")
            },
            Expression::Not(expr) => {
                write!(f, "{:>width$}\n","Not(");
                write!(f,"{:>width$},\n", expr, width = width + 2*TAB_WIDTH);
                write!(f, "{:>width$}\n",")")
            },
            Expression::Minus(expr) => {
                write!(f, "{:>width$}\n","Minus(");
                write!(f,"{:>width$},\n", expr, width = width + 2*TAB_WIDTH);
                write!(f, "{:>width$}\n",")")
            }
        };
        write!(f, "{:>width$}\n",')')

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
