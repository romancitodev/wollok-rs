use winnow::{Parser, combinator::alt};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Literal, Span, SpannedToken, Token},
};

pub struct BooleanParser;

impl TokenParser for BooleanParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        alt(("true".value(true), "false".value(false)))
            .with_span()
            .map(|(bool_val, span)| {
                Some(SpannedToken::new(
                    Span::from(span),
                    Token::Literal(Literal::Boolean(bool_val)),
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
    fn test_true_literal() {
        let mut input = Src::new("true");
        let result = BooleanParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!(true));
    }

    #[test]
    fn test_false_literal() {
        let mut input = Src::new("false");
        let result = BooleanParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!(false));
    }

    #[test]
    fn test_not_boolean() {
        let mut input = Src::new("maybe");
        let result = BooleanParser::parse(&mut input);

        assert!(result.is_err());
    }
}
