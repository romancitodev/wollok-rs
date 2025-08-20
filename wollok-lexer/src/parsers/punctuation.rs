use winnow::{Parser, combinator::alt};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Punctuation, Span, SpannedToken, Token},
};

pub struct PunctuationParser;

impl TokenParser for PunctuationParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        alt([
            // Operadores de dos caracteres primero
            "==".value(Punctuation::Eq),
            "!=".value(Punctuation::Ne),
            "&&".value(Punctuation::And),
            "||".value(Punctuation::Or),
            // Operadores aritméticos
            "+".value(Punctuation::Plus),
            "-".value(Punctuation::Minus),
            "*".value(Punctuation::Multiply),
            "/".value(Punctuation::Div),
            // // Operadores bitwise
            "|".value(Punctuation::BitOr),
            "&".value(Punctuation::BitAnd),
            // Operadores de un caracter
            "=".value(Punctuation::Equals),
            "!".value(Punctuation::Bang),
            ",".value(Punctuation::Comma),
            ";".value(Punctuation::Semicolon),
            ":".value(Punctuation::Colon),
            ".".value(Punctuation::Dot),
            "$".value(Punctuation::Dollar),
            // Delimitadores
            "{".value(Punctuation::OpenBrace),
            "}".value(Punctuation::CloseBrace),
            "(".value(Punctuation::OpenParen),
            ")".value(Punctuation::CloseParen),
            "[".value(Punctuation::OpenSquareBracket),
            "]".value(Punctuation::CloseSquareBracket),
            // Whitespace especial
            "\n".value(Punctuation::Newline),
            "\t".value(Punctuation::Identation),
        ])
        .with_span()
        .map(|(punct, span)| {
            Some(SpannedToken::new(
                Span::from(span),
                Token::Punctuation(punct),
            ))
        })
        .parse_next(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{T, error::Src};

    #[test]
    fn test_equals_operator() {
        let mut input = Src::new("==");
        let result = PunctuationParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token.token, T!(Eq));
    }

    #[test]
    fn test_single_equals() {
        let mut input = Src::new("=");
        let result = PunctuationParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token.token, T!(Equals));
    }

    #[test]
    fn test_open_brace() {
        let mut input = Src::new("{");
        let result = PunctuationParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token.token, T!(OpenBrace));
    }
}
