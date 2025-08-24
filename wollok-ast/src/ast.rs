use tracing::{debug, info, trace, warn};

use wollok_lexer::{
    lexer::TokenStream,
    macros::{T, kw},
    token::Token,
};

use crate::{
    expr::{Expr, ExprArray, ExprLit, ExprSet},
    item::{Item, ItemConst, ItemLet, ItemObject, ItemProperty},
    source::Ast,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Scope(pub Vec<Stmt>);

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

    fn parse_pre_statement(&mut self) -> bool {
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

    fn parse_set(&mut self) -> Expr {
        _ = self.expect_token(&T!(OpenBrace));

        self.parse_collection("set", &T!(CloseBrace), |elements| {
            Expr::Set(ExprSet { elements })
        })
    }

    fn parse_array(&mut self) -> Expr {
        self.parse_collection("array", &T!(CloseSquareBracket), |elements| {
            Expr::Array(ExprArray { elements })
        })
    }

    fn parse_expr(&mut self) -> Expr {
        // this is the other side of the =
        let primitive = self.expect();
        trace!("Parsing expression starting with token: {:?}", *primitive);
        match *primitive {
            Token::Literal(ref lit) => Expr::Lit(ExprLit { value: lit.clone() }),
            T!(OpenSquareBracket) => self.parse_array(),
            T!(Hash) => self.parse_set(),
            _ => self.error_in_place("Expected expression"),
        }
    }

    fn parse_item(&mut self) -> Item {
        let item = self.expect();
        debug!("Parsing item: {:?}", *item);
        match *item {
            kw!(Const) => {
                trace!("Parsing const declaration");
                let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
                self.expect_token(&T!(Equals));
                let expr = Box::new(self.parse_expr());
                debug!("Parsed const '{}' with expression", name);
                Item::Const(ItemConst { name, expr })
            }
            kw!(Let) => {
                trace!("Parsing let declaration");
                let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
                self.expect_token(&T!(Equals));
                let expr = Box::new(self.parse_expr());
                debug!("Parsed let '{}' with expression", name);
                Item::Let(ItemLet { name, expr })
            }
            kw!(Property) => {
                trace!("Parsing property declaration");
                let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
                self.expect_token(&T!(Equals));
                let expr = Box::new(self.parse_expr());
                debug!("Parsed property '{}' with expression", name);
                Item::Property(ItemProperty { name, expr })
            }
            kw!(Method) => {
                warn!("Method parsing not yet implemented");
                todo!()
            }
            _ => {
                warn!("Unexpected token in item parsing: {:?}", *item);
                self.error_in_place(format!("Unexpected token in item parsing: {:?}", *item))
            }
        }
    }

    fn parse_statement(&mut self) -> Stmt {
        let token = self.peek_expect();
        match **token {
            wollok_lexer::token::Token::Comment(_) => {
                unreachable!("Comments should be handled in parse_pre_statement")
            }
            wollok_lexer::token::Token::Ident(_) => todo!(),
            wollok_lexer::token::Token::Punctuation(_) => todo!(),
            wollok_lexer::token::Token::Literal(_) => todo!(),
            kw!(Object) => self.parse_object(),
            Token::Keyword(kw!(@raw Let) | kw!(@raw Const)) => {
                token.recover();
                Stmt::Item(self.parse_item())
            }
            wollok_lexer::token::Token::Keyword(_) => todo!(),
        }
    }

    fn parse_object_body(&mut self) -> Vec<Item> {
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

            let stmt = self.parse_item();
            Self::push_to_node(stmt, &mut body);
        }

        body
    }

    fn parse_object(&mut self) -> Stmt {
        trace!("Starting object parsing");
        let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
        debug!("Parsing object '{}'", name);
        self.expect_token(&T!(OpenBrace)); // Here we should expect the `{`
        let body = self.parse_object_body();
        self.expect_token(&T!(CloseBrace)); // Here we should expect the `}`
        info!(
            "Successfully parsed object '{}' with {} items",
            name,
            body.len()
        );

        Stmt::Item(Item::Object(ItemObject { name, body }))
    }

    fn push_to_node<T>(stmt: T, nodes: &mut Vec<T>) {
        // For the moment, we just do:
        // INFO: But, maybe in a future we might use `match stmt` and then classify the stmt.
        nodes.push(stmt);
    }

    fn parse_element_expr(&mut self) -> Expr {
        // Parse a single expression element (for arrays, sets, etc.)
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

    fn parse_collection(
        &mut self,
        collection_name: &str,
        close_token: &Token,
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
