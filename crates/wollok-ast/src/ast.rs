use std::fmt::Display;

use tracing::{debug, info, trace};

use wollok_lexer::{
    lexer::TokenStream,
    macros::{T, cmt},
};

use crate::{expr::Expr, item::Item, source::Ast};

#[derive(Debug, Clone, PartialEq)]
pub struct Scope(pub Vec<Stmt>);

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope with {} statements", self.0.len())?;
        for stmt in &self.0 {
            writeln!(f, "{stmt}")?;
        }
        Ok(())
    }
}

impl std::ops::Deref for Scope {
    type Target = Vec<Stmt>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Item(Item),
    Expr(Expr),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Item(item) => write!(f, "{item}"),
            Stmt::Expr(expr) => write!(f, "{expr}"),
        }
    }
}

impl Scope {
    #[must_use]
    pub fn from_tokens(base: &str, tokens: TokenStream<'_>) -> Self {
        info!("Starting AST parsing from tokens for base: {}", base);
        let result = Ast::new(base, tokens).parse_scope();
        info!(
            "Successfully parsed AST scope with {} statements",
            result.0.len()
        );
        result
    }
}

impl Ast<'_> {
    /// Parses the entire scope (top-level statements and declarations)
    pub(crate) fn parse_scope(&mut self) -> Scope {
        debug!("Parsing scope");
        let mut nodes = vec![];

        loop {
            if !self.parse_pre_statement() {
                trace!("No more pre-statements to parse, ending scope");
                break;
            }

            trace!("Parsing statement in scope");
            let stmt = self.parse_statement();
            Self::push_to_node(stmt, &mut nodes);
        }

        debug!("Completed scope parsing with {} nodes", nodes.len());
        Scope(nodes)
    }

    /// Handles pre-statement parsing (newlines, comments, etc.)
    /// Returns false when no more tokens are available
    pub(crate) fn parse_pre_statement(&mut self) -> bool {
        loop {
            let Some(first) = self.peek() else {
                return false;
            };

            match **first {
                T![Newline] => {
                    _ = first.accept();
                }
                wollok_lexer::token::Token::Comment(_) => {
                    // Consume and ignore comment tokens
                    _ = first.accept();
                    trace!("Skipped comment token in pre-statement");
                }
                _ => {
                    first.recover();
                    return true;
                }
            }
        }
    }

    /// Skips comment tokens in the token stream
    pub(crate) fn skip_comments(&mut self) {
        while let Some(token) = self.peek() {
            if matches!(**token, cmt!(@match _)) {
                trace!("skipping comment");
                _ = token.accept();
            } else {
                token.recover();
                break;
            }
        }
    }

    pub(crate) fn push_to_node<T>(stmt: T, nodes: &mut Vec<T>) {
        nodes.push(stmt);
    }
}
