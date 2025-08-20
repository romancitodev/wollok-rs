use wollok_common::ast::AstExpr;

#[derive(Debug, Clone, PartialEq)]
pub struct AstScope(pub Vec<AstStatement>);

#[derive(Debug, Clone, PartialEq)]
pub enum AstStatement {
    Comment,
    Expression(Box<AstExpr>),
    Conditional {
        test: Box<AstExpr>,
        body: AstScope,
        otherwise: Option<AstScope>,
    },
}
