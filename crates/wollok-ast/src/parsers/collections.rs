//! Collection parsing utilities
//!
//! This module contains all collection-related parsing logic, including:
//! - Array expressions ([1, 2, 3])
//! - Set expressions (#{1, 2, 3})
//! - Generic collection parsing with custom delimiters

use tracing::debug;
use wollok_lexer::macros::T;

use crate::{
    expr::{Expr, ExprArray, ExprSet},
    source::Ast,
};

impl Ast<'_> {
    /// Parses array expressions starting with [
    pub(crate) fn parse_array(&mut self) -> Expr {
        debug!("Parsing array expression");
        let elements =
            self.parse_separated_list(Ast::parse_element_expr, &T!(Comma), &T!(CloseSquareBracket));
        debug!("Parsed array with {} elements", elements.len());
        Expr::Array(ExprArray { elements })
    }

    /// Parses set expressions starting with #
    pub(crate) fn parse_set(&mut self) -> Expr {
        debug!("Parsing set expression");
        self.expect_token(&T!(OpenBrace));
        let elements =
            self.parse_separated_list(Ast::parse_element_expr, &T!(Comma), &T!(CloseBrace));
        debug!("Parsed set with {} elements", elements.len());
        Expr::Set(ExprSet { elements })
    }
}
