use wollok_lexer::{lexer::TokenStream, macros::T};

use crate::{expr::Expr, item::Item, source::Ast};

#[derive(Debug, Clone, PartialEq)]
pub struct Scope(pub Vec<Stmt>);

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Item(Item),
    Expr(Expr),
}

impl Scope {
    #[must_use]
    pub fn from_tokens(base: &str, tokens: TokenStream<'_>) -> Self {
        Ast::new(base, tokens).parse_scope(0)
    }
}

impl Ast<'_> {
    pub(crate) fn parse_scope(&mut self, ident: usize) -> Scope {
        let mut nodes = vec![];

        loop {
            if !self.parse_pre_statement(ident) {
                break;
            }

            let stmt = self.parse_statement(ident);
            Self::push_to_node(stmt, &mut nodes);
        }

        Scope(nodes)
    }

    fn parse_pre_statement(&mut self, ident: usize) -> bool {
        let Some(first) = self.peek() else {
            return false;
        };

        if *first != T![Newline] {
            first.recover();
            return true;
        }

        loop {
            if ident != 0 {
                match self.eat_ident(ident) {
                    Some(false) => {}
                    Some(true) => continue,
                    None => return false,
                }

                let Some(first) = self.peek() else {
                    return false;
                };

                if *first != T![Newline] {
                    first.recover();
                    return true;
                }
            }
        }
    }

    fn parse_statement(&mut self, ident: usize) -> Stmt {
        todo!()
    }

    fn eat_ident(&mut self, ident: usize) -> Option<bool> {
        for _ in 0..ident {
            let token = self.peek()?;

            match *token.token {
                T![Identation] => {}
                T![Newline] => return Some(true),
                _ => {
                    token.recover();
                    return None;
                }
            }
        }

        Some(false)
    }

    fn push_to_node(stmt: Stmt, nodes: &mut Vec<Stmt>) {
        // For the moment, we just do:
        // INFO: But, maybe in a future we might use `match stmt` and then classify the stmt.
        nodes.push(stmt);
    }
}
