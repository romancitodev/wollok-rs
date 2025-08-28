//! Collection parsing utilities
//!
//! This module contains all collection-related parsing logic, including:
//! - Array expressions ([1, 2, 3])
//! - Set expressions (#{1, 2, 3})
//! - Generic collection parsing with custom delimiters

use tracing::debug;
use wollok_lexer::{macros::T, token::Token};

use crate::{
    expr::{Expr, ExprArray, ExprSet},
    source::Ast,
};

impl Ast<'_> {
    /// Parses array expressions starting with [
    pub(crate) fn parse_array(&mut self) -> Expr {
        self.parse_collection("array", &T!(CloseSquareBracket), |elements| {
            Expr::Array(ExprArray { elements })
        })
    }

    /// Parses set expressions starting with #
    pub(crate) fn parse_set(&mut self) -> Expr {
        _ = self.expect_token(&T!(OpenBrace));

        self.parse_collection("set", &T!(CloseBrace), |elements| {
            Expr::Set(ExprSet { elements })
        })
    }

    /// Generic parser for collections (arrays, sets) with custom delimiters
    pub(crate) fn parse_collection<'t>(
        &mut self,
        collection_name: &'t str,
        close_token: &'t Token,
        create_expr: impl Fn(Vec<Expr>) -> Expr,
    ) -> Expr {
        debug!("Parsing {} expression", collection_name);
        let mut elements = Vec::new();

        if let Some(token) = self.peek() {
            if **token == *close_token {
                _ = token.accept();
                debug!("Parsed empty {}", collection_name);
                return create_expr(elements);
            }
            // Not empty, put the token back
            token.recover();
        }

        // Parse collection elements
        loop {
            // Parse the next expression (element inside the collection)
            debug!("About to parse {} element", collection_name);
            let element_expr = self.parse_element_expr();
            elements.push(element_expr);
            debug!(
                "Parsed {} element, now checking what follows",
                collection_name
            );

            // Check what follows the expression
            if let Some(token) = self.peek() {
                debug!("Found token after element: {:?}", **token);
                match **token {
                    ref t if *t == T!(Comma) => {
                        // Consume comma and continue to next element
                        debug!("Found comma, consuming it");
                        _ = token.accept();

                        // Check for trailing comma before closing delimiter
                        if let Some(next_token) = self.peek() {
                            debug!("After comma, next token is: {:?}", **next_token);
                            if **next_token == *close_token {
                                debug!("Found trailing comma before closing delimiter, breaking");
                                next_token.recover();
                                break;
                            }
                            debug!("After comma, continuing to next element");
                            next_token.recover();
                        } else {
                            debug!("No token after comma - unexpected end");
                        }
                    }
                    ref t if *t == *close_token => {
                        // End of collection - put the token back so expect_token can consume it
                        token.recover();
                        break;
                    }
                    _ => {
                        let unexpected = token.accept();
                        self.error_at(
                            unexpected.span,
                            format!(
                                "Expected ',' or closing delimiter, found {:?}",
                                unexpected.token
                            ),
                        );
                    }
                }
            } else {
                // Unexpected end of input (this will panic)
                self.error_in_place(format!(
                    "Unexpected end of input while parsing {collection_name}"
                ));
            }
        }

        self.expect_token(close_token);
        debug!(
            "Parsed {} with {} elements",
            collection_name,
            elements.len()
        );
        create_expr(elements)
    }
}
