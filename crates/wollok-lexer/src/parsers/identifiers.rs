use winnow::{
    Parser,
    combinator::peek,
    error::ParserError,
    token::{any, take_while},
};

use crate::{
    error::{LexerErr, Result, Src},
    parsers::TokenParser,
    token::{Span, SpannedToken, Token},
};

pub struct IdentifierParser;

impl TokenParser for IdentifierParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        // Primero verificamos que el primer carácter no sea un dígito
        let (first_char, _) = peek(any).with_span().parse_next(input)?;

        if first_char.is_ascii_digit() {
            return Err(LexerErr::from_input(input));
        }

        let (ident, span) = take_while(1.., |c: char| c.is_alphanumeric() || c == '_')
            .with_span()
            .parse_next(input)?;

        if ident.is_empty() {
            return Ok(None);
        }

        Ok(Some(SpannedToken::new(
            Span::from(span),
            Token::Ident(ident.to_owned()),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Src, ident};

    #[test]
    fn test_simple_identifier() {
        let mut input = Src::new("variable");
        let result = IdentifierParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, ident!("variable"));
    }

    #[test]
    fn test_identifier_with_underscore() {
        let mut input = Src::new("_private_var");
        let result = IdentifierParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, ident!("_private_var"));
    }

    #[test]
    fn test_identifier_with_numbers() {
        let mut input = Src::new("var123");
        let result = IdentifierParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, ident!("var123"));
    }

    #[test]
    fn test_not_identifier_starts_with_number() {
        let mut input = Src::new("123var");
        let result = IdentifierParser::parse(&mut input);

        assert!(result.is_err());
    }
}
