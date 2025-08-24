use crate::ast::Scope;
use wollok_lexer::lexer::TokenStream;

fn parse(input: &'_ str) -> Scope {
    Scope::from_tokens(input, TokenStream::new(input))
}

#[cfg(test)]
mod ast {
    use crate::{
        ast::Stmt,
        expr::{Expr, ExprArray, ExprLit},
        item::{Item, ItemConst},
    };

    use super::*;

    #[test]
    fn test_array_parse() {
        let input = "const items = [1, 2, 3]";
        let scope = parse(input);
        println!("{scope:#?}");
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "items".into(),
                expr: Box::new(Expr::Array(ExprArray {
                    elements: vec![
                        Expr::Lit(ExprLit { value: 1.into() }),
                        Expr::Lit(ExprLit { value: 2.into() }),
                        Expr::Lit(ExprLit { value: 3.into() }),
                    ],
                })),
            }))]
        );
    }
}
