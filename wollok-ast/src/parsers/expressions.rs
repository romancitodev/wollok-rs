/// Expression parsing utilities
///
/// This module contains all expression-related parsing logic, including:
/// - Primary expressions (literals, identifiers, parentheses)
/// - Assignment expressions
/// - Field access expressions
use tracing::{debug, trace};
use wollok_common::ast::{BinaryOp, UnaryOp};
use wollok_lexer::{
    macros::{T, kw},
    token::Token,
};

use crate::{
    ast::Stmt,
    expr::{
        Block, Expr, ExprAssign, ExprBinary, ExprCall, ExprClosure, ExprField, ExprIf, ExprLit,
        ExprReturn, ExprUnary,
    },
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
        let expr = self.parse_unary_expr();
        self.parse_binary_expr(expr, 0)
    }

    /// Parses unary expressions (e.g. `!condition`)
    pub(crate) fn parse_unary_expr(&mut self) -> Expr {
        if self.consume(&T!(Bang)) {
            return Expr::Unary(ExprUnary {
                op: UnaryOp::Not,
                expr: Box::new(self.parse_unary_expr()),
            });
        }
        self.parse_postfix_expr()
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
            // self()/super() call the current/parent constructor
            Expr::Self_ |
            Expr::Super_
        )
    }

    /// Parses atomic expressions without binary operations
    pub(crate) fn parse_atomic_expr(&mut self) -> Expr {
        let token = self.expect();
        trace!("Parsing atomic expression with token: {:?}", *token);
        self.skip_comments();

        match *token {
            Token::Ident(ref ident) => {
                // `ident => body` is a single-param closure with no parens
                if self.consume(&T!(FatArrow)) {
                    Expr::Closure(ExprClosure {
                        params: vec![ident.clone()],
                        body: Box::new(self.parse_expr()),
                    })
                } else {
                    Expr::Field(ExprField {
                        name: ident.clone(),
                        base: Box::new(Expr::Self_),
                    })
                }
            }
            kw!(New) => {
                let name = self.expect_match("Expected class name", |t| t.into_ident());
                let params = self.parse_params();
                Expr::Class(crate::expr::ExprClass { name, params })
            }
            kw!(This) => Expr::Self_,
            kw!(Super) => Expr::Super_,
            kw!(If) => self.parse_if_expr(),
            kw!(Return) => {
                let value = (!self.check(&T!(Newline)) && !self.check(&T!(CloseBrace)))
                    .then(|| Box::new(self.parse_expr()));
                Expr::Return(ExprReturn { value })
            }
            Token::Literal(ref lit) => Expr::Lit(ExprLit { value: lit.clone() }),
            T!(OpenSquareBracket) => self.parse_array(),
            T!(Hash) => self.parse_set(),
            T!(OpenParen) => {
                if self.peek_is_closure_params() {
                    self.parse_closure_after_params()
                } else {
                    self.parse_parenthesized_expr()
                }
            }
            T!(OpenBrace) => self.parse_brace_closure_expr(),
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
            let rhs_atomic = self.parse_unary_expr();
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
                T!(Pow) => Some((BinaryOp::Pow, 6, true)),
                T!(Multiply) => Some((BinaryOp::Multiply, 5, false)),
                T!(Div) => Some((BinaryOp::Div, 5, false)),
                T!(Modulo) => Some((BinaryOp::Modulo, 5, false)),
                T!(Plus) => Some((BinaryOp::Plus, 4, false)),
                T!(Minus) => Some((BinaryOp::Minus, 4, false)),
                T!(Lt) => Some((BinaryOp::Lt, 3, false)),
                T!(Le) => Some((BinaryOp::Le, 3, false)),
                T!(Gt) => Some((BinaryOp::Gt, 3, false)),
                T!(Ge) => Some((BinaryOp::Ge, 3, false)),
                T!(Eq) => Some((BinaryOp::Eq, 2, false)),
                T!(Ne) => Some((BinaryOp::Ne, 2, false)),
                T!(And) => Some((BinaryOp::And, 1, false)),
                T!(Or) => Some((BinaryOp::Or, 0, false)),
                _ => None,
            };
            peeked.recover();
            result
        })
    }

    /// Parses an `if (cond) then else otherwise` expression, where each
    /// branch can be a `{ block }` or a single inline expression.
    pub(crate) fn parse_if_expr(&mut self) -> Expr {
        debug!("Parsing if expression");
        self.expect_token(&T!(OpenParen));
        let condition = Box::new(self.parse_expr());
        self.expect_token(&T!(CloseParen));

        let then = self.parse_branch_block();
        let otherwise = self.consume(&kw!(Else)).then(|| self.parse_branch_block());

        Expr::If(ExprIf {
            condition,
            then,
            otherwise,
        })
    }

    /// Parses one branch of an `if`/`else`: either a `{ ... }` block or a
    /// single expression treated as a one-statement block.
    fn parse_branch_block(&mut self) -> Block {
        if self.consume(&T!(OpenBrace)) {
            let block = self.parse_block();
            self.expect_token(&T!(CloseBrace));
            block
        } else {
            Block {
                stmts: vec![Stmt::Expr(self.parse_expr())],
            }
        }
    }

    /// Looks ahead (without consuming) past an already-consumed `(` to
    /// check whether it opens a closure signature, i.e. `)` or
    /// `ident (, ident)* )` immediately followed by `=>`.
    fn peek_is_closure_params(&self) -> bool {
        let mut idx = 0;
        match self.tokens.get(idx).map(|t| &t.token) {
            Some(T!(CloseParen)) => idx += 1,
            Some(Token::Ident(_)) => {
                idx += 1;
                loop {
                    match self.tokens.get(idx).map(|t| &t.token) {
                        Some(T!(Comma)) => {
                            idx += 1;
                            match self.tokens.get(idx).map(|t| &t.token) {
                                Some(Token::Ident(_)) => idx += 1,
                                _ => return false,
                            }
                        }
                        Some(T!(CloseParen)) => {
                            idx += 1;
                            break;
                        }
                        _ => return false,
                    }
                }
            }
            _ => return false,
        }
        matches!(self.tokens.get(idx).map(|t| &t.token), Some(T!(FatArrow)))
    }

    /// Parses `params) => body` right after the opening `(`, once
    /// `peek_is_closure_params` confirmed the shape.
    fn parse_closure_after_params(&mut self) -> Expr {
        let params = self.parse_separated_list(
            |p| p.expect_match("Expected parameter name", |t| t.into_ident()),
            &T!(Comma),
            &T!(CloseParen),
        );
        self.expect_token(&T!(FatArrow));
        Expr::Closure(ExprClosure {
            params,
            body: Box::new(self.parse_expr()),
        })
    }

    /// Parses a `{ params => body }` closure literal (comma-separated
    /// params, no parens; body is a single expression).
    fn parse_brace_closure_expr(&mut self) -> Expr {
        let params = self.parse_separated_list(
            |p| p.expect_match("Expected parameter name", |t| t.into_ident()),
            &T!(Comma),
            &T!(FatArrow),
        );
        let body = Box::new(self.parse_expr());
        self.expect_token(&T!(CloseBrace));
        Expr::Closure(ExprClosure { params, body })
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
