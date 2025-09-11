use owo_colors::OwoColorize;
use std::fmt::Display;

use wollok_common::ast::{BinaryOp, UnaryOp};
use wollok_lexer::{macros::lit, token::Literal};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Expr {
    Array(ExprArray),
    Set(ExprSet),
    Assign(ExprAssign),
    Binary(ExprBinary),
    // Block(ExprBlock),
    Call(ExprCall),
    Closure(ExprClosure),
    Const(ExprConst),
    Field(ExprField),
    Class(ExprClass),
    // ForLoop(ExprForLoop),
    // Group(ExprGroup),
    If(ExprIf),
    // Index(ExprIndex),
    Let(ExprLet),
    Lit(ExprLit),
    // Loop(ExprLoop),
    // Match(ExprMatch),
    MethodCall(ExprMethodCall),
    Object(ExprObject),
    Paren(ExprParen),
    // Path(ExprPath),
    Return(ExprReturn),
    Try(ExprTry),
    TryBlock(ExprTryBlock),
    Tuple(ExprTuple),
    Unary(ExprUnary),
    // While(ExprWhile),
    Self_,
    Super(ExprSuper),
    New(ExprNew),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprArray {
    pub elements: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprSet {
    pub elements: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAssign {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprBinary {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub op: BinaryOp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

// A lambda expression / closure en Wollok: { param1, param2 => body }
#[derive(Debug, Clone, PartialEq)]
pub struct ExprClosure {
    pub params: Vec<String>, // nombres de parámetros
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprConst {
    pub block: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprField {
    pub base: Box<Expr>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprIf {
    pub condition: Box<Expr>,
    pub then: Block,
    pub otherwise: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprReturn {
    pub value: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprLet {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprLit {
    pub value: Literal, // This could be an enum for different literal types
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprMethodCall {
    pub receiver: Box<Expr>, // objeto al que se le envía el mensaje
    pub name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprParen {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprTry {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprTryBlock {
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprTuple {
    pub elements: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprUnary {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

// Expresiones específicas de Wollok

#[derive(Debug, Clone, PartialEq)]
pub struct ExprSuper {
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprNew {
    pub class_name: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprObject {
    pub fields: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprClass {
    pub name: String,
    pub params: Vec<Expr>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: &dyn Display = match self {
            Expr::Array(expr) => expr,
            Expr::Set(expr) => expr,
            Expr::Assign(expr) => expr,
            Expr::Binary(expr) => expr,
            Expr::Call(expr) => expr,
            Expr::Closure(expr) => expr,
            Expr::Const(expr) => expr,
            Expr::Field(expr) => expr,
            Expr::Class(expr) => expr,
            Expr::If(expr) => expr,
            Expr::Let(expr) => expr,
            Expr::Lit(expr) => expr,
            Expr::MethodCall(expr) => expr,
            Expr::Object(expr) => expr,
            Expr::Paren(expr) => expr,
            Expr::Return(expr) => expr,
            Expr::Try(expr) => expr,
            Expr::TryBlock(expr) => expr,
            Expr::Tuple(expr) => expr,
            Expr::Unary(expr) => expr,
            Expr::Self_ => &"Self",
            Expr::Super(expr) => expr,
            Expr::New(expr) => expr,
        };
        write!(f, "{v}")
    }
}

impl Display for ExprArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        _ = write!(f, "{}", "[".green());
        for el in &self.elements {
            _ = write!(f, "{el}");
            _ = write!(f, ", ");
        }
        write!(f, "{}", "]".green())
    }
}

impl Display for ExprSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        _ = write!(f, "{}", "#{{".green());
        for el in &self.elements {
            _ = write!(f, "{el}");
            _ = write!(f, "{}", ", ".black());
        }
        write!(f, "{}", "#}}".green())
    }
}

impl Display for ExprAssign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lhs = self.left.as_ref();
        let rhs = self.right.as_ref();
        _ = write!(f, "{lhs}");
        _ = write!(f, "{}", " = ".black());
        write!(f, "{rhs}")
    }
}

impl Display for ExprBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lhs = self.left.as_ref();
        let op = &self.op;
        let rhs = self.right.as_ref();
        _ = write!(f, "{lhs}");
        _ = write!(f, " {op} ");
        write!(f, "{rhs}")
    }
}

impl Display for ExprCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let call = self.callee.as_ref().white().to_string();
        let args = &self.args;

        _ = write!(f, "{call}(");
        for (i, arg) in args.iter().enumerate() {
            _ = write!(f, "{arg}");
            if i < args.len() - 1 {
                _ = write!(f, ", ");
            }
        }
        write!(f, ")")
    }
}

impl Display for ExprClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "{ ".yellow())?;
        for (i, param) in self.params.iter().enumerate() {
            write!(f, "{}", param.cyan())?;
            if i < self.params.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "{}", " => ".yellow())?;
        write!(f, "{}", self.body)?;
        write!(f, "{}", " }".yellow())
    }
}

impl Display for ExprConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "const ".magenta(), self.block)
    }
}

impl Display for ExprField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.base.as_ref() {
            Expr::Self_ => write!(f, "{}", self.name.blue()),
            _ => write!(f, "{}.{}", self.base, self.name.blue()),
        }
    }
}

impl Display for ExprClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "new ".magenta(), self.name.cyan())?;
        write!(f, "(")?;
        for (i, param) in self.params.iter().enumerate() {
            write!(f, "{param}")?;
            if i < self.params.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Display for ExprIf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "if ".magenta(), self.condition)?;
        write!(f, " {{ ... }}")?; // Simplified display for blocks
        if let Some(else_expr) = &self.otherwise {
            write!(f, "{}{}", " else ".magenta(), else_expr)?;
        }
        Ok(())
    }
}

impl Display for ExprLet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            "var ".magenta(),
            self.name.cyan(),
            " = ".black(),
            self.value
        )
    }
}

impl Display for ExprLit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Literal::String(s) => {
                write!(f, "{}", format!("\"{s}\"").green())
            }
            Literal::Integer(n) => write!(f, "{}", n.to_string().yellow()),
            Literal::Float(n) => write!(f, "{}", n.to_string().yellow()),
            Literal::Boolean(b) => write!(f, "{}", b.to_string().red()),
            Literal::Null => write!(f, "{}", "null".red()),
        }
    }
}

impl Display for ExprMethodCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.receiver, self.name.blue())?;
        write!(f, "(")?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{arg}")?;
            if i < self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Display for ExprObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "object { ".magenta())?;
        for field in &self.fields {
            write!(f, "{field} ")?;
        }
        write!(f, "{}", "}".magenta())
    }
}

impl Display for ExprParen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.expr)
    }
}

impl Display for ExprReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "return".magenta())?;
        if let Some(value) = &self.value {
            write!(f, " {value}")?;
        }
        Ok(())
    }
}

impl Display for ExprTry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "try ".magenta(), self.expr)
    }
}

impl Display for ExprTryBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "try { ... }".magenta()) // Simplified display for blocks
    }
}

impl Display for ExprTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, element) in self.elements.iter().enumerate() {
            write!(f, "{element}")?;
            if i < self.elements.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Display for ExprUnary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.op, self.expr)
    }
}

impl Display for ExprSuper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "super".magenta())?;
        if !self.args.is_empty() {
            write!(f, "(")?;
            for (i, arg) in self.args.iter().enumerate() {
                write!(f, "{arg}")?;
                if i < self.args.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl Display for ExprNew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", "new ".magenta(), self.class_name.cyan())?;
        write!(f, "(")?;
        for (i, arg) in self.args.iter().enumerate() {
            write!(f, "{arg}")?;
            if i < self.args.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "{ ".white())?;
        for stmt in &self.stmts {
            write!(f, "{stmt}; ")?;
        }
        write!(f, "{}", "}".white())
    }
}
