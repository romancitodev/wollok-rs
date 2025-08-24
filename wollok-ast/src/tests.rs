#[cfg(test)]
mod ast {

    #[must_use]
    fn parse(input: &'_ str) -> Scope {
        Scope::from_tokens(input, TokenStream::new(input))
    }

    use wollok_lexer::lexer::TokenStream;

    use crate::{
        ast::{Scope, Stmt},
        expr::{Expr, ExprArray, ExprLit},
        item::{Item, ItemConst},
    };

    #[test]
    fn test_array_parse() {
        let input = "const items = [1, 2, 3]";
        let scope = parse(input);
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

    #[test]
    fn test_literal_parse() {
        let input = "const value = 42";
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "value".into(),
                expr: Box::new(Expr::Lit(ExprLit { value: 42.into() })),
            }))]
        );
    }
}
