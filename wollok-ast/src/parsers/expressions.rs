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
        ExprReturn, ExprTry, ExprUnary,
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
            kw!(Try) => Expr::Try(ExprTry {
                expr: Box::new(self.parse_expr()),
            }),
            kw!(Return) => {
                let value = (!self.check(&T!(Newline)) && !self.check(&T!(CloseBrace)))
                    .then(|| Box::new(self.parse_expr()));
                Expr::Return(ExprReturn { value })
            }
            Token::Literal(ref lit) => Expr::Lit(ExprLit { value: lit.clone() }),
            T!(OpenSquareBracket) => self.parse_array(),
            T!(Hash) => self.parse_set(),
            T!(OpenParen) => match self.optional(Self::try_parse_closure_signature) {
                Some(params) => Expr::Closure(ExprClosure {
                    params,
                    body: Box::new(self.parse_expr()),
                }),
                None => self.parse_parenthesized_expr(),
            },
            T!(OpenBrace) => self.parse_brace_closure(),
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

    /// None means "not a closure signature" (e.g. `(1 + 2)`); optional()
    /// rolls back whatever this consumed.
    fn try_parse_closure_signature(&mut self) -> Option<Vec<String>> {
        let mut params = Vec::new();

        match self.try_match(|t| match t.token {
            T!(CloseParen) => Some(None),
            Token::Ident(name) => Some(Some(name)),
            _ => None,
        })? {
            None => return self.closure_signature_if_arrow_follows(params),
            Some(name) => params.push(name),
        }

        loop {
            match self.try_match(|t| match t.token {
                T!(Comma) => Some(true),
                T!(CloseParen) => Some(false),
                _ => None,
            })? {
                false => break,
                true => params.push(self.try_match(|t| t.token.into_ident())?),
            }
        }
        self.closure_signature_if_arrow_follows(params)
    }

    fn closure_signature_if_arrow_follows(&mut self, params: Vec<String>) -> Option<Vec<String>> {
        self.try_match(|t| matches!(t.token, T!(FatArrow)).then_some(()))?;
        Some(params)
    }

    fn parse_brace_closure(&mut self) -> Expr {
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
