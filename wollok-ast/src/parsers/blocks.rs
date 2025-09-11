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

use crate::{ast::Stmt, expr::Block, source::Ast};

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
            let stmt = self.parse_statement();
            stmts.push(stmt);
        }

        Block { stmts }
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
        stmts.push(Stmt::Expr(stmt));

        Block { stmts }
    }

    /// Parses a single statement (can be a local declaration or an expression)
    /// Only allows `const` and `let` declarations, not `property` (which is class/object level)
    pub(crate) fn parse_statement(&mut self) -> Stmt {
        let token = self.peek_expect();
        match **token {
            kw!(Object) => self.parse_object(),
            kw!(Class) => self.parse_class(),
            Token::Keyword(kw!(@raw Let) | kw!(@raw Const)) => {
                token.recover();
                Stmt::Item(self.parse_item())
            }
            _ => {
                // If it's not a declaration keyword, try to parse it as an expression
                token.recover();
                let expr = self.parse_expr();
                Stmt::Expr(expr)
            }
        }
    }
}
