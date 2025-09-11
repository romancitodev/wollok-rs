/// Expression parsing utilities
///
/// This module contains all expression-related parsing logic, including:
/// - Primary expressions (literals, identifiers, parentheses)
/// - Assignment expressions
/// - Field access expressions
use tracing::{debug, trace};
use wollok_common::ast::BinaryOp;
use wollok_lexer::{
    macros::{T, kw},
    token::Token,
};

use crate::{
    expr::{Expr, ExprAssign, ExprBinary, ExprCall, ExprField, ExprLit},
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
        if self.consume(&T!(Equals)) {
            let value = Box::new(self.parse_expr());
            // Create assignment expression
            return Expr::Assign(ExprAssign {
                left: Box::new(expr),
                right: value,
            });
        }

        expr
    }

    /// Parses primary expressions (literals, identifiers, collections, etc.)
    pub(crate) fn parse_primary_expr(&mut self) -> Expr {
        let expr = self.parse_postfix_expr();
        self.parse_binary_expr(expr, 0)
    }

    /// Parses postfix expressions (function calls, field access, etc.)
    /// Only allows calls on callable expressions (identifiers, field access, parentheses)
    pub(crate) fn parse_postfix_expr(&mut self) -> Expr {
        let mut expr = self.parse_atomic_expr();

        loop {
            if self.check(&T!(OpenParen)) && Self::is_callable(&expr) {
                // Function call: expr() - but only if expr is callable
                let args = self.parse_params();
                expr = Expr::Call(ExprCall {
                    callee: Box::new(expr),
                    args,
                });
            } else if self.check(&T!(Dot)) {
                // Field access: expr.field
                self.advance(); // consume the dot
                let field_name = self.expect_match("Expected field name", |t| t.into_ident());
                expr = Expr::Field(ExprField {
                    name: field_name,
                    base: Box::new(expr),
                });
            } else {
                // No more postfix operations
                break;
            }
        }

        expr
    }

    /// Determines if an expression can be called (i.e., can have () after it)
    fn is_callable(expr: &Expr) -> bool {
        matches!(
            expr,
            // Identifiers can be called: foo()
            Expr::Field(_) |
            // Method calls can be chained: obj.method1().method2()
            Expr::Call(_) |
            // Object instantiation can be called: new Foo().method()
            Expr::Class(_) |
            // Self can be called: self()
            Expr::Self_
        )
    }

    /// Parses atomic expressions without binary operations
    pub(crate) fn parse_atomic_expr(&mut self) -> Expr {
        let token = self.expect();
        trace!("Parsing atomic expression with token: {:?}", *token);
        self.skip_comments();

        match *token {
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

    /// Parse binary expressions using precedence climbing
    pub(crate) fn parse_binary_expr(&mut self, mut lhs: Expr, min_prec: u8) -> Expr {
        while let Some((op, prec, right_assoc)) = self.peek_operator() {
            if prec < min_prec {
                break;
            }

            self.advance(); // consume operator
            let next_prec = if right_assoc { prec } else { prec + 1 };
            let rhs_atomic = self.parse_postfix_expr();
            let rhs = self.parse_binary_expr(rhs_atomic, next_prec);

            lhs = Expr::Binary(ExprBinary {
                left: Box::new(lhs),
                right: Box::new(rhs),
                op,
            });
        }
        lhs
    }

    /// Peek at the next operator, returning (`BinaryOp`, precedence)
    fn peek_operator(&mut self) -> Option<(BinaryOp, u8, bool)> {
        self.peek().and_then(|peeked| {
            let token = &peeked.token.token;
            let result = match token {
                T!(Multiply) => Some((BinaryOp::Multiply, 4, false)),
                T!(Div) => Some((BinaryOp::Div, 4, false)),
                T!(Plus) => Some((BinaryOp::Plus, 2, false)),
                T!(Minus) => Some((BinaryOp::Minus, 2, false)),
                _ => None,
            };
            peeked.recover();
            result
        })
    }

    /// Parses expressions enclosed in parentheses
    pub(crate) fn parse_parenthesized_expr(&mut self) -> Expr {
        debug!("Parsing the parenthized expr");
        let expr = self.parse_expr();
        self.expect_token(&T!(CloseParen));
        expr // For now, we just return the inner expression
    }

    /// Parses a single expression element (for arrays, sets, etc.)
    /// This is now unified with primary expression logic
    pub(crate) fn parse_element_expr(&mut self) -> Expr {
        self.parse_primary_expr()
    }
}
