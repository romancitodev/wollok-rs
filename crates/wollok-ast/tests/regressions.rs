//! Regressions found by running every `wollok` fenced code block in
//! docs/*.md through the real parser instead of trusting that "the feature
//! is done".

use wollok_ast::ast::Scope;
use wollok_lexer::lexer::TokenStream;

fn parse(input: &str) -> Scope {
  Scope::from_tokens(input, TokenStream::new(input))
}

#[test]
fn multiline_array_literal() {
  parse(
    r#"const tabla = [
            ["A", "B", "C"],
            ["1", "2", "3"],
        ]"#,
  );
}

#[test]
fn multiline_method_chain() {
  parse(
    r"const resultado = numeros
            .filter(n => n > 2)
            .map(n => n * 3)
            .fold(0, (sum, n) => sum + n)",
  );
}

#[test]
fn inline_body_on_next_line() {
  let scope = parse(
    r"object articulo {
            method tieneEtiqueta(etiqueta) =
                etiquetas.contains(etiqueta)
        }",
  );
  // The real bug: this used to silently produce an EMPTY method body
  // instead of erroring or parsing the expression.
  let text = scope.to_string();
  assert!(text.contains("contains"), "body was dropped: {text}");
}

#[test]
fn brace_closure_without_arrow() {
  parse("const r = {2 + 5}.apply()");
}

#[test]
fn closure_with_multi_statement_block_body() {
  parse(
    r"object contador {
            let valor = 0
            method incrementador() = () => {
                valor = valor + 1
                return valor
            }
        }",
  );
}
