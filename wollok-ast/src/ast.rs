use tracing::{debug, info, trace, warn};

use wollok_lexer::{
    lexer::TokenStream,
    macros::{T, kw},
    token::Token,
};

use crate::{
    expr::{Expr, ExprArray, ExprLit},
    item::{Item, ItemConst, ItemLet, ItemObject, ItemProperty},
    source::Ast,
};

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
        info!("Starting AST parsing from tokens for base: {}", base);
        let result = Ast::new(base, tokens).parse_scope(0);
        info!(
            "Successfully parsed AST scope with {} statements",
            result.0.len()
        );
        result
    }
}

impl Ast<'_> {
    pub(crate) fn parse_scope(&mut self, ident: usize) -> Scope {
        debug!("Parsing scope at indentation level: {}", ident);
        let mut nodes = vec![];

        loop {
            if !self.parse_pre_statement(ident) {
                trace!("No more pre-statements to parse, ending scope");
                break;
            }

            trace!("Parsing statement in scope");
            let stmt = self.parse_statement(ident);
            Self::push_to_node(stmt, &mut nodes);
        }

        debug!("Completed scope parsing with {} nodes", nodes.len());
        Scope(nodes)
    }

    fn parse_pre_statement(&mut self, ident: usize) -> bool {
        loop {
            let Some(first) = self.peek() else {
                return false;
            };

            match **first {
                T![Newline] => {
                    _ = first.accept();

                    if ident != 0 {
                        match self.eat_ident(ident) {
                            Some(false) => return true,
                            Some(true) => {}
                            None => return false,
                        }
                    }
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

    fn parse_array(&mut self, ident: usize) -> Expr {
        debug!("Parsing array expression");
        let mut elements = Vec::new();

        if let Some(token) = self.peek()
            && matches!(**token, T!(CloseSquareBracket))
        {
            _ = token.accept();
            debug!("Parsed empty array");
            return Expr::Array(ExprArray { elements });
        }

        // Parse array elements
        loop {
            // Parse the next expression
            let expr = self.parse_expr(ident);
            elements.push(expr);

            // Check what follows the expression
            if let Some(token) = self.peek() {
                match **token {
                    T![Comma] => {
                        // Consume comma and continue to next element
                        _ = token.accept();

                        // Check for trailing comma before closing bracket
                        if let Some(next_token) = self.peek()
                            && matches!(**next_token, T!(CloseSquareBracket))
                        {
                            break;
                        }
                    }
                    T![CloseSquareBracket] => {
                        // End of array
                        break;
                    }
                    _ => {
                        // Unexpected token - report error (this will panic)
                        let unexpected = token.accept();
                        self.error_at(
                            unexpected.span,
                            format!("Expected ',' or ']', found {:?}", unexpected.token),
                        );
                    }
                }
            } else {
                // Unexpected end of input (this will panic)
                self.error_in_place("Unexpected end of input while parsing array");
            }
        }

        self.expect_token(&T!(CloseSquareBracket));
        debug!("Parsed array with {} elements", elements.len());
        Expr::Array(ExprArray { elements })
    }

    fn parse_expr(&mut self, ident: usize) -> Expr {
        // this is the other side of the =
        let primitive = self.expect();
        trace!("Parsing expression starting with token: {:?}", *primitive);
        match *primitive {
            Token::Literal(ref lit) => Expr::Lit(ExprLit { value: lit.clone() }),
            T!(OpenSquareBracket) => self.parse_array(ident),
            _ => self.error_in_place("Expected expression"),
        }
    }

    fn parse_item(&mut self, ident: usize) -> Item {
        let item = self.expect();
        debug!("Parsing item: {:?}", *item);
        match *item {
            kw!(Const) => {
                trace!("Parsing const declaration");
                let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
                self.expect_token(&T!(Equals));
                let expr = Box::new(self.parse_expr(ident));
                debug!("Parsed const '{}' with expression", name);
                Item::Const(ItemConst { name, expr })
            }
            kw!(Let) => {
                trace!("Parsing let declaration");
                let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
                self.expect_token(&T!(Equals));
                let expr = Box::new(self.parse_expr(ident));
                debug!("Parsed let '{}' with expression", name);
                Item::Let(ItemLet { name, expr })
            }
            kw!(Property) => {
                trace!("Parsing property declaration");
                let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
                self.expect_token(&T!(Equals));
                let expr = Box::new(self.parse_expr(ident));
                debug!("Parsed property '{}' with expression", name);
                Item::Property(ItemProperty { name, expr })
            }
            kw!(Method) => {
                warn!("Method parsing not yet implemented");
                todo!()
            }
            _ => {
                warn!("Unexpected token in item parsing: {:?}", *item);
                self.error_in_place("error")
            }
        }
    }

    fn parse_statement(&mut self, ident: usize) -> Stmt {
        let token = self.peek_expect();
        match **token {
            wollok_lexer::token::Token::Comment(_) => {
                unreachable!("Comments should be handled in parse_pre_statement")
            }
            wollok_lexer::token::Token::Ident(_) => todo!(),
            wollok_lexer::token::Token::Punctuation(_) => todo!(),
            wollok_lexer::token::Token::Literal(_) => todo!(),
            kw!(Object) => self.parse_object(ident),
            wollok_lexer::token::Token::Keyword(_) => todo!(),
        }
    }

    fn parse_object_body(&mut self, ident: usize) -> Vec<Item> {
        let mut body = Vec::new();

        loop {
            let Some(first) = self.peek() else {
                break;
            };

            if *first == T![Newline] {
                // Consume the newline token and continue
                _ = first.accept();
                continue;
            }

            if *first == T![CloseBrace] {
                first.recover();
                break;
            }

            first.recover();

            let stmt = self.parse_item(ident);
            Self::push_to_node(stmt, &mut body);
        }

        body
    }

    fn parse_object(&mut self, ident: usize) -> Stmt {
        trace!("Starting object parsing");
        let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
        debug!("Parsing object '{}'", name);
        self.expect_token(&T!(OpenBrace)); // Here we should expect the `{`
        let body = self.parse_object_body(ident + 1);
        self.expect_token(&T!(CloseBrace)); // Here we should expect the `}`
        info!(
            "Successfully parsed object '{}' with {} items",
            name,
            body.len()
        );

        Stmt::Item(Item::Object(ItemObject { name, body }))
    }

    fn eat_ident(&mut self, ident: usize) -> Option<bool> {
        for _ in 0..ident {
            let token = self.peek()?;

            match **token {
                T![Identation] => {
                    // Consume the indentation token
                    _ = token.accept();
                }
                T![Newline] => return Some(true),
                _ => {
                    token.recover();
                    return None;
                }
            }
        }

        Some(false)
    }

    fn push_to_node<T>(stmt: T, nodes: &mut Vec<T>) {
        // For the moment, we just do:
        // INFO: But, maybe in a future we might use `match stmt` and then classify the stmt.
        nodes.push(stmt);
    }
}
