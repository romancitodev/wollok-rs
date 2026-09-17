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
  item::{
    Item, ItemClass, ItemConst, ItemImport, ItemLet, ItemMethod, ItemMixin, ItemObject,
    ItemPrefixedMethod, ItemProperty, Prefix, Signature,
  },
  source::Ast,
};

impl Ast<'_> {
  fn parse_override(&mut self) -> (Item, Prefix) {
    if self.consume(&kw!(Fallible)) {
      info!("Entering on fallible method");
      (self.parse_item(), Prefix::OverrideFallible)
    } else {
      (self.parse_item(), Prefix::Override)
    }
  }

  pub(crate) fn parse_class_item(&mut self) -> Item {
    self.skip_comments();
    if self.consume(&kw!(Override)) {
      info!("Entering on override item");
      let (item, prefix) = self.parse_override();
      let Item::Method(method) = item else {
        self.error_in_place("expected a method");
      };
      Item::PrefixedMethod(ItemPrefixedMethod { prefix, method })
    } else if self.consume(&kw!(Fallible)) {
      let Item::Method(method) = self.parse_item() else {
        self.error_in_place("expected a method");
      };
      Item::PrefixedMethod(ItemPrefixedMethod {
        prefix: Prefix::Fallible,
        method,
      })
    } else if self.consume(&kw!(Abstract)) {
      info!("Entering on abstract method");
      self.expect_token(&kw!(Method));
      let signature = self.parse_method_signature();
      Item::PrefixedMethod(ItemPrefixedMethod {
        prefix: Prefix::Abstract,
        method: ItemMethod {
          signature,
          body: None,
          inline: false,
        },
      })
    } else if self.consume(&kw!(Native)) {
      info!("Entering on native method");
      self.expect_token(&kw!(Method));
      let signature = self.parse_method_signature();
      Item::PrefixedMethod(ItemPrefixedMethod {
        prefix: Prefix::Native,
        method: ItemMethod {
          signature,
          body: None,
          inline: false,
        },
      })
    } else {
      self.parse_item()
    }
  }

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
          Item::Method(ItemMethod {
            signature,
            body: Some(body),
            inline: false,
          })
        } else if self.consume(&T!(Equals)) {
          let body = self.parse_inline_block();
          Item::Method(ItemMethod {
            signature,
            body: Some(body),
            inline: true,
          })
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
    let params = self.parse_separated_list(Ast::parse_expr, &T!(Comma), &T!(CloseParen));
    trace!("Parsed {} parameters", params.len());
    params
  }

  pub(crate) fn parse_import(&mut self) -> Stmt {
    let mut module = self.expect_match("Expected module name", |t| t.into_ident());
    let mut wildcard = false;

    while self.consume(&T!(Dot)) {
      if self.consume(&T!(Multiply)) {
        wildcard = true;
        break;
      }
      let part = self.expect_match("Expected module path segment", |t| t.into_ident());
      module.push('.');
      module.push_str(&part);
    }

    Stmt::Item(Item::Import(ItemImport { module, wildcard }))
  }

  /// Parses `keyword name, name, ...`, e.g. `inherits A, B` or
  /// `with M, N`. Returns `None` if `keyword` isn't there at all.
  fn parse_name_list_after(&mut self, keyword: &Token) -> Option<Vec<String>> {
    if !self.consume(keyword) {
      return None;
    }
    let mut names = vec![self.expect_match("Expected identifier", |t| t.into_ident())];
    while self.consume(&T!(Comma)) {
      if self.check(&T!(OpenBrace)) {
        let (span, _) = self.advance().unwrap().split();
        self.error_at(span, "Expected identifer, got , instead");
      }
      names.push(self.expect_match("Expected identifier", |t| t.into_ident()));
    }
    Some(names)
  }

  /// Parses a class declaration with its body
  pub(crate) fn parse_class(&mut self, is_abstract: bool) -> Stmt {
    trace!("Starting class parsing");
    let name = self.expect_match("Expected class identifier", |t| t.into_ident()); // Here we should expect the object ident.
    debug!("Parsing class '{}'", name);
    let superclass = self.parse_name_list_after(&kw!(Inherits));
    let mixins = self.parse_name_list_after(&kw!(With));
    self.expect_token(&T!(OpenBrace)); // Here we should expect the `{`
    self.skip_trivia();
    let body = self.parse_class_body();
    self.expect_token(&T!(CloseBrace)); // Here we should expect the `}`
    self.skip_trivia();
    info!(
      "Successfully parsed class '{}' with {} items",
      name,
      body.len()
    );

    Stmt::Item(Item::Class(ItemClass {
      name,
      superclass,
      mixins,
      body,
      is_abstract,
    }))
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

  /// Parses a mixin declaration with its body (methods, including
  /// abstract ones — a mixin body parses like a class body).
  pub(crate) fn parse_mixin(&mut self) -> Stmt {
    trace!("Starting mixin parsing");
    let name = self.expect_match("Expected mixin identifier", |t| t.into_ident());
    self.expect_token(&T!(OpenBrace));
    self.skip_trivia();
    let body = self.parse_class_body();
    self.expect_token(&T!(CloseBrace));
    self.skip_trivia();
    info!(
      "Successfully parsed mixin '{}' with {} items",
      name,
      body.len()
    );

    Stmt::Item(Item::Mixin(ItemMixin { name, body }))
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
      let stmt = self.parse_class_item();
      Self::push_to_node(stmt, &mut body);
    }

    body
  }

  /// Parses the body of a class (very similar to `parse_object_body`) (its properties, methods, etc.)
  pub(crate) fn parse_class_body(&mut self) -> Vec<Item> {
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
      let stmt = self.parse_class_item();
      Self::push_to_node(stmt, &mut body);
    }

    body
  }
}
