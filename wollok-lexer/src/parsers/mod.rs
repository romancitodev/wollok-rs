use crate::{
    error::{Result, Src},
    token::SpannedToken,
};

/// Trait común para todos los parsers de tokens
pub trait TokenParser {
    /// Intenta parsear un token desde la entrada.
    /// Retorna `Ok(Some(token))` si encuentra un token,
    /// `Ok(None)` si no encuentra nada (para que otro parser lo intente),
    /// `Err(...)` si hay un error de parsing.
    ///
    /// # Errors
    /// Retorna un error si la entrada no puede ser parseada correctamente como un token válido.
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>>;
}

/// Re-exports de todos los parsers
pub mod comments;
pub mod identifiers;
pub mod keywords;
pub mod literals;
pub mod operators;
pub mod punctuation;
pub mod whitespace;

pub use comments::CommentParser;
pub use identifiers::IdentifierParser;
pub use keywords::KeywordParser;
pub use literals::LiteralParser;
pub use operators::OperatorParser;
pub use punctuation::PunctuationParser;
pub use whitespace::WhitespaceParser;
