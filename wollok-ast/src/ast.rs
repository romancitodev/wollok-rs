use wollok_common::ast::Expr;

use crate::item::Item;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Item(Item),
    Expr(Expr),
}
