/// Item parsing utilities
///
/// This module contains all item-related parsing logic, including:
/// - Object declarations
/// - Method declarations and signatures
/// - Property declarations
/// - Const and let declarations
use tracing::{debug, info, trace, warn};
use wollok_lexer::{
    macros::{T, kw},
    token::Token,
};

use crate::{
    ast::Stmt,
    expr::Expr,
    item::{Ident, Item, ItemConst, ItemLet, ItemMethod, ItemObject, ItemProperty, Signature},
    source::Ast,
};

impl Ast<'_> {
    /// Parses items (const, let, property, method declarations)
    pub(crate) fn parse_item(&mut self) -> Item {
        self.skip_comments();
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
                trace!("Parsing method declaration");
                let signature = self.parse_method_signature();
                let token = self.peek_expect();
                if matches!(**token, T!(OpenBrace)) {
                    _ = token.accept(); // Consume the opening brace
                    let body = self.parse_block();
                    self.expect_token(&T!(CloseBrace));
                    return Item::Method(ItemMethod { signature, body });
                } else if matches!(**token, T!(Equals)) {
                    _ = token.accept();
                    let body = self.parse_inline_block();
                    return Item::Method(ItemMethod { signature, body });
                }
                self.error_in_place("Expected '{' or '=' after method signature");
            }
            _ => {
                warn!("Unexpected token in item parsing: {:?}", *item);
                self.error_in_place(format!("Unexpected token in item parsing: {:?}", *item))
            }
        }
    }

    /// Parses method signature including name and parameters
    pub(crate) fn parse_method_signature(&mut self) -> Signature {
        let mut params = Vec::new();

        let name = self.expect_match("Expected method identifier", |t| t.into_ident());
        self.expect_token(&T!(OpenParen));
        loop {
            let Some(token) = self.peek() else {
                self.error_in_place("Unexpected end of input in method signature");
            };

            let (span, parsed) = token.split();

            trace!("Parsed token in method signature: {:?}", parsed);
            match parsed {
                T!(CloseParen) => {
                    token.recover();
                    break;
                } // End of parameters
                T!(Comma) => {
                    _ = token.accept(); // Consume comma and continue
                }
                Token::Ident(ref ident) => {
                    let param_name = ident.clone();
                    trace!("Parsed parameter: {:?}", param_name);
                    params.push(Ident { name: param_name });
                }
                _ => {
                    self.error_at(
                        span,
                        format!("Unexpected token in method signature: {parsed:?}"),
                    );
                }
            }
        }

        self.expect_token(&T!(CloseParen));

        trace!(
            "Parsed method signature: {}({})",
            name,
            params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        );

        Signature {
            ident: name,
            params,
        }
    }

    pub(crate) fn parse_params(&mut self) -> Vec<Expr> {
        self.expect_token(&T!(OpenParen));
        let params = self.parse_separated_list(
            |parser| parser.parse_expr(),
            &T!(Comma),
            &T!(CloseParen),
        );
        trace!("Parsed {} parameters", params.len());
        params
    }

    /// Parses an object declaration with its body
    pub(crate) fn parse_object(&mut self) -> Stmt {
        trace!("Starting object parsing");
        let name = self.expect_match("Expected object identifier", |t| t.into_ident()); // Here we should expect the object ident.
        debug!("Parsing object '{}'", name);
        self.expect_token(&T!(OpenBrace)); // Here we should expect the `{`
        self.skip_comments();
        let body = self.parse_object_body();
        self.expect_token(&T!(CloseBrace)); // Here we should expect the `}`
        self.skip_comments();
        info!(
            "Successfully parsed object '{}' with {} items",
            name,
            body.len()
        );

        Stmt::Item(Item::Object(ItemObject { name, body }))
    }

    /// Parses the body of an object (its properties, methods, etc.)
    pub(crate) fn parse_object_body(&mut self) -> Vec<Item> {
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
}
