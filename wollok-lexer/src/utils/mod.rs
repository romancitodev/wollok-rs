use winnow::Parser;

/// Utilidades comunes para parsing
pub mod patterns;
pub mod unicode;

pub use patterns::*;
pub use unicode::*;

/// Helper para crear parsers que ignoran ciertos tokens
pub fn ignore_whitespace<'t, F, O>(
    mut parser: F,
) -> impl FnMut(&mut crate::error::Src<'t>) -> winnow::Result<O>
where
    F: Parser<crate::error::Src<'t>, O, winnow::error::ContextError>,
{
    move |input| {
        // Consumimos whitespace antes del parser
        let _ =
            winnow::token::take_while(0.., |c: char| c == ' ' || c == '\t').parse_next(input)?;

        parser.parse_next(input)
    }
}

/// Helper para verificar que un identificador no sea una keyword
#[must_use]
pub fn not_keyword(ident: &str) -> bool {
    !matches!(
        ident,
        "if" | "else" | "object" | "class" | "method" | "import" | "describe" | "assert"
    )
}
