use winnow::{
    Parser,
    combinator::{alt, delimited},
    token::take_while,
};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Literal, Span, SpannedToken, Token},
};

pub struct StringParser;

impl TokenParser for StringParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        alt((
            // String con comillas dobles
            delimited('"', take_while(0.., |c: char| c != '"' && c != '\n'), '"').map(|s: &str| s),
            // String con comillas simples
            delimited(
                '\'',
                take_while(0.., |c: char| c != '\'' && c != '\n'),
                '\'',
            )
            .map(|s: &str| s),
        ))
        .with_span()
        .map(|(content, span)| {
            let string_content = content.to_string();
            Some(SpannedToken::new(
                Span::from(span),
                Token::Literal(Literal::String(string_content)),
            ))
        })
        .parse_next(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Src, lit};

    #[test]
    fn test_double_quoted_string() {
        let mut input = Src::new("\"hello world\"");
        let result = StringParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!("hello world"));
    }

    #[test]
    fn test_single_quoted_string() {
        let mut input = Src::new("'hello world'");
        let result = StringParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!("hello world"));
    }

    #[test]
    fn test_empty_string() {
        let mut input = Src::new("\"\"");
        let result = StringParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!(""));
    }

    #[test]
    fn test_unclosed_string() {
        let mut input = Src::new("\"unclosed");
        let result = StringParser::parse(&mut input);

        assert!(result.is_err());
    }
}
