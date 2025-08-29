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
};

use crate::{
    ast::Stmt,
    expr::Expr,
    item::{Item, ItemConst, ItemLet, ItemMethod, ItemObject, ItemProperty, Signature},
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
                
                if self.consume(&T!(OpenBrace)) {
                    let body = self.parse_block();
                    self.expect_token(&T!(CloseBrace));
                    Item::Method(ItemMethod { signature, body })
                } else if self.consume(&T!(Equals)) {
                    let body = self.parse_inline_block();
                    Item::Method(ItemMethod { signature, body })
                } else {
                    self.error_in_place("Expected '{' or '=' after method signature");
                }
            }
            _ => {
                warn!("Unexpected token in item parsing: {:?}", *item);
                self.error_in_place(format!("Unexpected token in item parsing: {:?}", *item))
            }
        }
    }

    /// Parses method signature including name and parameters
    pub(crate) fn parse_method_signature(&mut self) -> Signature {
        let name = self.expect_match("Expected method identifier", |t| t.into_ident());
        self.expect_token(&T!(OpenParen));
        let params = self.parse_identifier_list(&T!(CloseParen));
        
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
        self.skip_trivia();
        let body = self.parse_object_body();
        self.expect_token(&T!(CloseBrace)); // Here we should expect the `}`
        self.skip_trivia();
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
            // Skip newlines
            if self.consume(&T!(Newline)) {
                continue;
            }

            // Check for end of object
            if self.check(&T!(CloseBrace)) {
                break;
            }

            // Parse item
            let stmt = self.parse_item();
            Self::push_to_node(stmt, &mut body);
        }

        body
    }
}
