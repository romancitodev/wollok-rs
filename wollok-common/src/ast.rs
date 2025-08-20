use std::fmt;
use wollok_lexer::token::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Literal(Literal),
    BinaryExpr {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryExpr {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Eq,  // ==
    Ne,  // !=
    And, // &&
    Or,  // ||
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
}

impl From<i64> for Expr {
    fn from(val: i64) -> Self {
        Expr::Literal(Literal::Integer(val))
    }
}

impl From<f64> for Expr {
    fn from(val: f64) -> Self {
        Expr::Literal(Literal::Float(val))
    }
}

impl From<&str> for Expr {
    fn from(val: &str) -> Self {
        Expr::Literal(Literal::String(val.to_owned()))
    }
}

impl From<String> for Expr {
    fn from(val: String) -> Self {
        Expr::Literal(Literal::String(val))
    }
}

impl From<bool> for Expr {
    fn from(val: bool) -> Self {
        Expr::Literal(Literal::Boolean(val))
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "!"),
        }
    }
}
