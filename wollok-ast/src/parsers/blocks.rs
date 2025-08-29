/// Block and statement parsing utilities
///
/// This module contains all block-related parsing logic, including:
/// - Method body blocks
/// - Inline blocks (single expressions)
/// - Statement parsing within blocks
/// - Top-level statement parsing
use tracing::trace;
use wollok_lexer::{
    macros::{T, kw},
    token::Token,
};

use crate::{
    ast::Stmt,
    expr::{Block, Expr},
    source::Ast,
};

impl Ast<'_> {
    /// Parses a method body block enclosed in braces, handling statements and expressions
    pub(crate) fn parse_block(&mut self) -> Block {
        trace!("Parsing block");
        let mut stmts = Vec::new();

        loop {
            // Skip whitespace and comments
            if !self.parse_pre_statement() {
                break;
            }

            // Check for block end
            if self.check(&T!(CloseBrace)) {
                break;
            }

            // Parse statement or expression
            let stmt = self.parse_block_statement();
            stmts.push(stmt);
        }

        Block { stmts }
    }

    /// Parses a single statement inside a block (assignment, declaration, or expression)
    pub(crate) fn parse_block_statement(&mut self) -> Expr {
        let token = self.peek_expect();

        match **token {
            // Handle const and let declarations inside blocks
            kw!(Const) => {
                token.recover(); // Put the token back
                let item = self.parse_item();
                match item {
                    crate::item::Item::Const(const_item) => Expr::Const(crate::expr::ExprConst {
                        block: const_item.expr,
                    }),
                    _ => unreachable!("parse_item with const keyword should return ItemConst"),
                }
            }
            kw!(Let) => {
                token.recover(); // Put the token back
                let item = self.parse_item();
                match item {
                    crate::item::Item::Let(let_item) => Expr::Let(crate::expr::ExprLet {
                        name: let_item.name,
                        value: let_item.expr,
                    }),
                    _ => unreachable!("parse_item with let keyword should return ItemLet"),
                }
            }
            // Handle identifiers (could be assignment or expression)
            Token::Ident(_) => {
                token.recover();
                // Parse as assignment expression
                self.parse_assignment_expr()
            }
            _ => {
                // Handle other expressions
                token.recover();
                self.parse_expr()
            }
        }
    }

    /// Parses a single expression inside an inline method body (method = expr)
    pub(crate) fn parse_inline_block(&mut self) -> Block {
        trace!("Parsing inline block");
        let mut stmts = Vec::new();

        // Check if we hit a newline (end of inline block)
        if self.check(&T!(Newline)) {
            trace!("Empty inline block");
            return Block { stmts };
        }

        // Parse single expression
        let stmt = self.parse_expr();
        trace!("Parsed statement: {:?}", stmt);
        stmts.push(stmt);

        Block { stmts }
    }

    /// Parses a single statement at the top level (objects, global declarations)
    pub(crate) fn parse_statement(&mut self) -> Stmt {
        let token = self.peek_expect();
        match **token {
            kw!(Object) => self.parse_object(),
            Token::Keyword(kw!(@raw Let) | kw!(@raw Const)) => {
                token.recover();
                Stmt::Item(self.parse_item())
            }
            _ => {
                self.error_in_place("Unexpected token in statement");
            }
        }
    }
}
