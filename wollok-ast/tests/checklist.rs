//! One smoke test per item on the "Terminar el Parsing" checklist
//! (github issue #1). Each test just needs to parse without panicking —
//! this file exists to answer "how much of the language do we actually
//! parse", not to assert exact ASTs (see tests/ast.rs for that).

use wollok_ast::ast::Scope;
use wollok_lexer::lexer::TokenStream;

fn parse(input: &str) -> Scope {
    Scope::from_tokens(input, TokenStream::new(input))
}

#[test]
fn self_reference() {
    parse("object foo { method run() = self }");
}

#[test]
fn inheritance() {
    parse("class Hijo inherits Base { }");
}

#[test]
fn super_call() {
    parse(
        r#"class Hijo inherits Base {
            override method saludo() = super.saludo() + "!"
            method ctor() = super()
        }"#,
    );
}

#[test]
fn abstract_methods() {
    parse(
        r"abstract class Figura {
            abstract method area()
            method mostrarInfo() { return area() }
        }",
    );
}

#[test]
fn fallible_methods() {
    parse("class C { fallible method riesgoso() = false }");
}

#[test]
fn advanced_arithmetic_operators() {
    parse("const a = 2 ** 3 % 2");
}

#[test]
fn comparison_operators() {
    parse("const a = 1 == 2 && 3 != 4 || 5 < 6 && 7 <= 8 && 9 > 10 && 11 >= 12");
}

#[test]
fn logical_operators() {
    parse("const a = true && !false || true");
}

#[test]
fn classes() {
    parse("class Punto { property x = 0 property y = 0 }");
}

#[test]
fn if_expression() {
    parse(r#"const mensaje = if (18 >= 18) "Adulto" else "Menor""#);
}

#[test]
fn if_else_if_chain_with_blocks() {
    parse(
        r#"object o {
            method evaluar(n) {
                if (n > 0) {
                    return "positivo"
                } else if (n < 0) {
                    return "negativo"
                } else {
                    return "cero"
                }
            }
        }"#,
    );
}

#[test]
fn try_expression() {
    parse("const resultado = try operacionRiesgosa()");
}

#[test]
fn closures() {
    parse(
        r"const a = n => n * 2
        const b = (x, y) => x + y
        const c = () => 0
        const d = numeros.filter({ n => n.even() })
        const e = numeros.fold(0, { acc, n => acc + n })",
    );
}

#[test]
fn collection_iteration_methods() {
    parse("const r = numeros.map({ n => n * 2 }).filter({ n => n > 0 })");
}

#[test]
fn collection_conversions() {
    parse("const r = [1, 2, 2, 3].asSet().asList()");
}

#[test]
#[ignore = "block comments /* */ not implemented yet"]
fn block_comments() {
    parse("/* hola */ const a = 1");
}

#[test]
#[ignore = "imports not implemented yet"]
fn imports() {
    parse("import trenes.*");
}

#[test]
#[ignore = "mixins not implemented yet"]
fn mixins() {
    parse("mixin Volador { method volar() = \"volando\" }");
}
