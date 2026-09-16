use owo_colors::OwoColorize;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Eq,  // ==
    Ne,  // !=
    And, // &&
    Or,  // ||

    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=

    Plus,     // +
    Minus,    // -
    Multiply, // *
    Div,      // /
    Modulo,   // %
    Pow,      // **
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Div => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Pow => "**",
        }
        .bright_red()
        .to_string();
        write!(f, "{op}")
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            UnaryOp::Not => "!",
        }
        .bright_red()
        .to_string();
        write!(f, "{op}")
    }
}
