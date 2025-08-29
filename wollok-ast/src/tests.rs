#[cfg(test)]
mod ast {

    #[must_use]
    fn parse(input: &'_ str) -> Scope {
        Scope::from_tokens(input, TokenStream::new(input))
    }

    use wollok_lexer::lexer::TokenStream;

    use crate::{
        ast::{Scope, Stmt},
        expr::{Block, Expr, ExprArray, ExprAssign, ExprClass, ExprField, ExprLit},
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
        let input = r"object calculator {
            method add(a, b) = 42
        }";
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

    #[test]
    fn test_class_expr() {
        let input = "const bar = new Foo()";
        let scope = parse(input);

        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "bar".into(),
                expr: Box::new(Expr::Class(ExprClass {
                    name: "Foo".into(),
                    params: vec![]
                }))
            }))]
        );
    }

    #[test]
    fn test_class_with_args() {
        let input = "const bar = new Foo(1, 2)";
        let scope = parse(input);

        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "bar".into(),
                expr: Box::new(Expr::Class(ExprClass {
                    name: "Foo".into(),
                    params: vec![
                        Expr::Lit(ExprLit { value: 1.into() }),
                        Expr::Lit(ExprLit { value: 2.into() }),
                    ]
                }))
            }))]
        );
    }

    #[test]
    fn test_complex_args() {
        let input = "const bar = new Foo(new Bar(), 2)";
        let scope = parse(input);

        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "bar".into(),
                expr: Box::new(Expr::Class(ExprClass {
                    name: "Foo".into(),
                    params: vec![
                        Expr::Class(ExprClass {
                            name: "Bar".into(),
                            params: vec![]
                        }),
                        Expr::Lit(ExprLit { value: 2.into() }),
                    ]
                }))
            }))]
        );
    }

    #[test]
    fn test_empty_array() {
        let input = "const empty = []";
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "empty".into(),
                expr: Box::new(Expr::Array(ExprArray { elements: vec![] })),
            }))]
        );
    }

    #[test]
    fn test_empty_set() {
        let input = "const empty = #{}";
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "empty".into(),
                expr: Box::new(Expr::Set(crate::expr::ExprSet { elements: vec![] })),
            }))]
        );
    }

    #[test]
    fn test_mixed_type_array() {
        let input = r#"const mixed = [1, "hello", true]"#;
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "mixed".into(),
                expr: Box::new(Expr::Array(ExprArray {
                    elements: vec![
                        Expr::Lit(ExprLit { value: 1.into() }),
                        Expr::Lit(ExprLit {
                            value: "hello".into()
                        }),
                        Expr::Lit(ExprLit { value: true.into() }),
                    ],
                })),
            }))]
        );
    }

    #[test]
    fn test_mixed_type_set() {
        let input = r#"const mixed = #{1, "hello", false}"#;
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "mixed".into(),
                expr: Box::new(Expr::Set(crate::expr::ExprSet {
                    elements: vec![
                        Expr::Lit(ExprLit { value: 1.into() }),
                        Expr::Lit(ExprLit {
                            value: "hello".into()
                        }),
                        Expr::Lit(ExprLit {
                            value: false.into()
                        }),
                    ],
                })),
            }))]
        );
    }

    #[test]
    fn test_nested_mixed_collections() {
        let input = r#"const complex = [#{1, 2}, [3, 4], "text"]"#;
        let scope = parse(input);
        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "complex".into(),
                expr: Box::new(Expr::Array(ExprArray {
                    elements: vec![
                        Expr::Set(crate::expr::ExprSet {
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
                        Expr::Lit(ExprLit {
                            value: "text".into()
                        }),
                    ],
                })),
            }))]
        );
    }

    #[test]
    fn test_method_no_params() {
        let input = r#"object counter {
            method increment() = 1
        }"#;
        let scope = parse(input);

        if let [Stmt::Item(Item::Object(obj))] = scope.as_slice() {
            assert_eq!(obj.name, "counter");
            if let [Item::Method(method)] = obj.body.as_slice() {
                assert_eq!(method.signature.ident, "increment");
                assert_eq!(method.signature.params.len(), 0);
            } else {
                panic!("Expected one method in object body");
            }
        } else {
            panic!("Expected object statement");
        }
    }

    #[test]
    fn test_method_single_param() {
        let input = r#"object calculator {
            method double(x) = 42
        }"#;
        let scope = parse(input);

        if let [Stmt::Item(Item::Object(obj))] = scope.as_slice() {
            assert_eq!(obj.name, "calculator");
            if let [Item::Method(method)] = obj.body.as_slice() {
                assert_eq!(method.signature.ident, "double");
                assert_eq!(method.signature.params.len(), 1);
                assert_eq!(method.signature.params[0].name, "x");
            } else {
                panic!("Expected one method in object body");
            }
        } else {
            panic!("Expected object statement");
        }
    }

    #[test]
    fn test_method_many_params() {
        let input = r#"object calculator {
            method calculate(a, b, c, d, e) = 42
        }"#;
        let scope = parse(input);

        if let [Stmt::Item(Item::Object(obj))] = scope.as_slice() {
            assert_eq!(obj.name, "calculator");
            if let [Item::Method(method)] = obj.body.as_slice() {
                assert_eq!(method.signature.ident, "calculate");
                assert_eq!(method.signature.params.len(), 5);
                assert_eq!(method.signature.params[0].name, "a");
                assert_eq!(method.signature.params[1].name, "b");
                assert_eq!(method.signature.params[2].name, "c");
                assert_eq!(method.signature.params[3].name, "d");
                assert_eq!(method.signature.params[4].name, "e");
            } else {
                panic!("Expected one method in object body");
            }
        } else {
            panic!("Expected object statement");
        }
    }

    #[test]
    fn test_object_with_multiple_items() {
        let input = r#"object complex {
            const value = 42
            let mutable = 0
        }"#;
        let scope = parse(input);

        if let [Stmt::Item(Item::Object(obj))] = scope.as_slice() {
            assert_eq!(obj.name, "complex");
            assert_eq!(obj.body.len(), 2);

            // Check const
            if let Item::Const(const_item) = &obj.body[0] {
                assert_eq!(const_item.name, "value");
            } else {
                panic!("Expected const item");
            }

            // Check let
            if let Item::Let(let_item) = &obj.body[1] {
                assert_eq!(let_item.name, "mutable");
            } else {
                panic!("Expected let item");
            }
        } else {
            panic!("Expected object statement");
        }
    }

    #[test]
    fn test_nested_object_instantiation() {
        let input = "const nested = new Outer(new Inner(1, 2), new Another())";
        let scope = parse(input);

        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "nested".into(),
                expr: Box::new(Expr::Class(ExprClass {
                    name: "Outer".into(),
                    params: vec![
                        Expr::Class(ExprClass {
                            name: "Inner".into(),
                            params: vec![
                                Expr::Lit(ExprLit { value: 1.into() }),
                                Expr::Lit(ExprLit { value: 2.into() }),
                            ]
                        }),
                        Expr::Class(ExprClass {
                            name: "Another".into(),
                            params: vec![]
                        }),
                    ]
                }))
            }))]
        );
    }

    #[test]
    fn test_class_with_collection_params() {
        let input = "const instance = new DataContainer([1, 2, 3], #{4, 5, 6})";
        let scope = parse(input);

        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "instance".into(),
                expr: Box::new(Expr::Class(ExprClass {
                    name: "DataContainer".into(),
                    params: vec![
                        Expr::Array(ExprArray {
                            elements: vec![
                                Expr::Lit(ExprLit { value: 1.into() }),
                                Expr::Lit(ExprLit { value: 2.into() }),
                                Expr::Lit(ExprLit { value: 3.into() }),
                            ],
                        }),
                        Expr::Set(crate::expr::ExprSet {
                            elements: vec![
                                Expr::Lit(ExprLit { value: 4.into() }),
                                Expr::Lit(ExprLit { value: 5.into() }),
                                Expr::Lit(ExprLit { value: 6.into() }),
                            ],
                        }),
                    ]
                }))
            }))]
        );
    }

    #[test]
    fn test_deeply_nested_collections() {
        let input = "const deep = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]";
        let scope = parse(input);

        assert_eq!(
            *scope,
            vec![Stmt::Item(Item::Const(ItemConst {
                name: "deep".into(),
                expr: Box::new(Expr::Array(ExprArray {
                    elements: vec![
                        Expr::Array(ExprArray {
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
                        }),
                        Expr::Array(ExprArray {
                            elements: vec![
                                Expr::Array(ExprArray {
                                    elements: vec![
                                        Expr::Lit(ExprLit { value: 5.into() }),
                                        Expr::Lit(ExprLit { value: 6.into() }),
                                    ],
                                }),
                                Expr::Array(ExprArray {
                                    elements: vec![
                                        Expr::Lit(ExprLit { value: 7.into() }),
                                        Expr::Lit(ExprLit { value: 8.into() }),
                                    ],
                                }),
                            ],
                        }),
                    ],
                })),
            }))]
        );
    }

    #[test]
    fn test_parenthesized_expressions() {
        let input = "const value = (42)";
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
    fn test_multiple_const_declarations() {
        let input = r#"
        const first = 1
        const second = "hello"
        const third = [1, 2, 3]
        "#;
        let scope = parse(input);

        assert_eq!(scope.len(), 3);

        if let [
            Stmt::Item(Item::Const(first)),
            Stmt::Item(Item::Const(second)),
            Stmt::Item(Item::Const(third)),
        ] = scope.as_slice()
        {
            assert_eq!(first.name, "first");
            assert_eq!(second.name, "second");
            assert_eq!(third.name, "third");
        } else {
            panic!("Expected three const declarations");
        }
    }

    #[test]
    fn test_let_declarations() {
        let input = r#"
        let mutable = 42
        let anotherMutable = "text"
        "#;
        let scope = parse(input);

        assert_eq!(scope.len(), 2);

        if let [Stmt::Item(Item::Let(first)), Stmt::Item(Item::Let(second))] = scope.as_slice() {
            assert_eq!(first.name, "mutable");
            assert_eq!(second.name, "anotherMutable");
        } else {
            panic!("Expected two let declarations");
        }
    }

    #[test]
    fn test_property_declarations() {
        let input = r#"object example {
            property count = 0
            property name = "default"
        }"#;
        let scope = parse(input);

        assert_eq!(scope.len(), 1);

        if let [Stmt::Item(Item::Object(obj))] = scope.as_slice() {
            assert_eq!(obj.name, "example");
            assert_eq!(obj.body.len(), 2);

            if let [Item::Property(first), Item::Property(second)] = obj.body.as_slice() {
                assert_eq!(first.name, "count");
                assert_eq!(second.name, "name");
            } else {
                panic!("Expected two property declarations");
            }
        } else {
            panic!("Expected object statement");
        }
    }

    #[test]
    fn test_complex_assignment_in_method() {
        let input = r#"object manager {
            let data = []
            method updateData() {
                data = [1, 2, 3]
            }
        }"#;
        let scope = parse(input);

        if let [Stmt::Item(Item::Object(obj))] = scope.as_slice() {
            assert_eq!(obj.name, "manager");
            assert_eq!(obj.body.len(), 2);

            if let [Item::Let(_), Item::Method(method)] = obj.body.as_slice() {
                assert_eq!(method.signature.ident, "updateData");
                assert_eq!(method.body.stmts.len(), 1);

                // Verify the assignment is to an array
                if let Expr::Assign(assign) = &method.body.stmts[0] {
                    if let Expr::Array(_) = assign.right.as_ref() {
                        // Good, assignment to array
                    } else {
                        panic!("Expected assignment to array");
                    }
                } else {
                    panic!("Expected assignment expression");
                }
            } else {
                panic!("Expected let and method items");
            }
        } else {
            panic!("Expected object statement");
        }
    }
}
