use wollok_common::ast::{BinaryOp, UnaryOp};
use wollok_lexer::token::Literal;

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
