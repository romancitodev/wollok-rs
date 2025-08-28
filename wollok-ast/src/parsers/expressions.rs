/// Expression parsing utilities
///
/// This module contains all expression-related parsing logic, including:
/// - Primary expressions (literals, identifiers, parentheses)
/// - Assignment expressions
/// - Field access expressions
use tracing::trace;
use wollok_lexer::{
    macros::{T, kw},
    token::Token,
};

use crate::{
    expr::{Expr, ExprAssign, ExprField, ExprLit},
    source::Ast,
};

impl Ast<'_> {
    /// Parses a complete expression, handling assignments and primary expressions
    pub(crate) fn parse_expr(&mut self) -> Expr {
        self.parse_assignment_expr()
    }

    /// Parses assignment expressions (e.g., `variable = value`)
    pub(crate) fn parse_assignment_expr(&mut self) -> Expr {
        let expr = self.parse_primary_expr();

        // Check if this is an assignment
        if let Some(token) = self.peek() {
            if matches!(**token, T!(Equals)) {
                _ = token.accept();
                let value = Box::new(self.parse_expr());
                // Create assignment expression
                return Expr::Assign(ExprAssign {
                    left: Box::new(expr),
                    right: value,
                });
            }
            token.recover();
        }

        expr
    }

    /// Parses primary expressions (literals, identifiers, collections, etc.)
    pub(crate) fn parse_primary_expr(&mut self) -> Expr {
        let primitive = self.expect();
        trace!("Parsing primary expression with token: {:?}", *primitive);
        self.skip_comments();

        match *primitive {
            Token::Ident(ref ident) => Expr::Field(ExprField {
                name: ident.clone(),
                base: Box::new(Expr::Self_),
            }),
            kw!(New) => {
                let name = self.expect_match("Expected class name", |t| t.into_ident());
                let params = self.parse_params();
                Expr::Class(crate::expr::ExprClass { name, params })
            }
            Token::Literal(ref lit) => Expr::Lit(ExprLit { value: lit.clone() }),
            T!(OpenSquareBracket) => self.parse_array(),
            T!(Hash) => self.parse_set(),
            T!(OpenParen) => self.parse_parenthesized_expr(),
            _ => self.error_in_place("Expected expression"),
        }
    }

    /// Parses expressions enclosed in parentheses
    pub(crate) fn parse_parenthesized_expr(&mut self) -> Expr {
        let expr = self.parse_expr();
        self.expect_token(&T!(CloseParen));
        expr // For now, we just return the inner expression
    }

    /// Parses a single expression element (for arrays, sets, etc.)
    pub(crate) fn parse_element_expr(&mut self) -> Expr {
        let primitive = self.expect();
        trace!(
            "Parsing element expression starting with token: {:?}",
            *primitive
        );
        match *primitive {
            Token::Literal(ref lit) => Expr::Lit(ExprLit { value: lit.clone() }),
            T!(OpenSquareBracket) => self.parse_array(), // Nested arrays
            T!(Hash) => self.parse_set(),                // Nested sets
            _ => self.error_in_place("Expected element expression"),
        }
    }
}
