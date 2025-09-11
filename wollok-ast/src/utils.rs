#[macro_export]
macro_rules! stmt {
    (@item $body:expr) => {
        $crate::ast::Stmt::Item($body)
    };
    (@expr $body:expr) => {
        $crate::ast::Stmt::Expr($body)
    };
}

#[macro_export]
macro_rules! ident {
    ($name:expr) => {
        $crate::item::Ident {
            name: $name.to_owned(),
        }
    };
}

#[macro_export]
macro_rules! items {
    ($item:ident, $body:expr) => {
        $crate::item::Item::$item($body)
    };
    (@const $name:expr, $value:expr) => {
        $crate::item::Item::Const($crate::item::ItemConst {
            name: $name.to_owned(),
            expr: $value,
        })
    };
    (@let $name:expr, $value:expr) => {
        $crate::item::Item::Let($crate::item::ItemLet {
            name: $name.to_owned(),
            expr: $value,
        })
    };
    (@object $name:expr, [$($body:expr),*]) => {
        $crate::item::Item::Object($crate::item::ItemObject {
            name: $name.to_owned(),
            body: vec![$($body),*],
        })
    };
    (@method $name:expr, $params:expr, $body:expr, $inline:expr) => {
        $crate::item::Item::Method($crate::item::ItemMethod {
            signature: $crate::item::Signature {
                ident: $name.to_owned(),
                params: $params,
              },
              body: $crate::expr::Block { stmts: $body },
              inline: $inline,
        })
    };
}

#[macro_export]
macro_rules! exprs {
  (@array [$($elements:expr),*]) => {
    $crate::expr::Expr::Array($crate::expr::ExprArray {
      elements: vec![$($elements),*],
    })
  };
  (@set [$($elements:expr),*]) => {
    $crate::expr::Expr::Set($crate::expr::ExprSet {
      elements: vec![$($elements),*],
    })
  };
  (@lit $value:expr) => {
    $crate::expr::Expr::Lit($crate::expr::ExprLit { value: $value.into() })
  };
  (@self) => {
    $crate::expr::Expr::Self_
  };
  (@class $name:expr, $params:expr) => {
    $crate::expr::Expr::Class($crate::expr::ExprClass {
      name: $name.to_owned(),
      params: $params,
    })
  };
  (@assign $left:expr, $right:expr) => {
    $crate::expr::Expr::Assign($crate::expr::ExprAssign {
      left: Box::new($left),
      right: Box::new($right),
    })
  };
  (@field $name:expr, $base:expr) => {
    $crate::expr::Expr::Field($crate::expr::ExprField {
      name: $name.to_owned(),
      base: Box::new($base),
    })
  };
}

// macro_rules! array {
//     ([]) => {
//         crate::expr::ExprArray {
//             elements: array!(@inner $elements),
//         }
//     };
// }
