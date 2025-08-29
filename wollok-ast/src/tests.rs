#[cfg(test)]
mod ast {

    #[must_use]
    fn parse(input: &'_ str) -> Scope {
        Scope::from_tokens(input, TokenStream::new(input))
    }

    use wollok_lexer::lexer::TokenStream;

    use crate::{
        ast::{Scope, Stmt},
        expr::{Block, Expr, ExprArray, ExprAssign, ExprField, ExprLit},
        item::{Item, ItemConst, ItemLet, ItemMethod, ItemObject, Signature},
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

    #[test]
    fn test_object_with_assignment_in_method() {
        let input = r"object foo {
  let a = 1
  method do() {
    a = 2
  }
}";
        let scope = parse(input);
        assert_eq!(
            scope.0,
            vec![Stmt::Item(Item::Object(ItemObject {
                name: "foo".into(),
                body: vec![
                    Item::Let(ItemLet {
                        name: "a".into(),
                        expr: Box::new(Expr::Lit(ExprLit { value: 1.into() })),
                    }),
                    Item::Method(ItemMethod {
                        signature: Signature {
                            ident: "do".into(),
                            params: vec![],
                        },
                        body: Block {
                            stmts: vec![Expr::Assign(ExprAssign {
                                left: Box::new(Expr::Field(ExprField {
                                    name: "a".into(),
                                    base: Box::new(Expr::Self_)
                                })),
                                right: Box::new(Expr::Lit(ExprLit { value: 2.into() }))
                            })],
                        }
                    }),
                ],
            }))]
        );
    }

    #[test]
    fn test_set_parse() {
        let input = "const numbers = #{1, 2, 3}";
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "numbers".into(),
                expr: Box::new(Expr::Set(crate::expr::ExprSet {
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
    fn test_nested_collections() {
        let input = "const nested = [[1, 2], [3, 4]]";
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "nested".into(),
                expr: Box::new(Expr::Array(ExprArray {
                    elements: vec![
                        Expr::Array(ExprArray {
                            elements: vec![
                                Expr::Lit(ExprLit { value: 1.into() }),
                                Expr::Lit(ExprLit { value: 2.into() }),
                            ],
                        }),
                        Expr::Array(ExprArray {
                            elements: vec![
                                Expr::Lit(ExprLit { value: 3.into() }),
                                Expr::Lit(ExprLit { value: 4.into() }),
                            ],
                        }),
                    ],
                })),
            }))]
        );
    }

    #[test]
    fn test_method_signature_parsing() {
        let input = r#"object calculator {
            method add(a, b) = 42
        }"#;
        let scope = parse(input);
        
        // Extract the object to check its method signature
        if let [Stmt::Item(Item::Object(obj))] = scope.as_slice() {
            assert_eq!(obj.name, "calculator");
            if let [Item::Method(method)] = obj.body.as_slice() {
                assert_eq!(method.signature.ident, "add");
                assert_eq!(method.signature.params.len(), 2);
                assert_eq!(method.signature.params[0].name, "a");
                assert_eq!(method.signature.params[1].name, "b");
            } else {
                panic!("Expected one method in object body");
            }
        } else {
            panic!("Expected object statement");
        }
    }
}
