use winnow::{Parser, combinator::alt};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Punctuation, Span, SpannedToken, Token},
};

pub struct OperatorParser;

impl TokenParser for OperatorParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        // Por ahora los operadores están en punctuation
        // Aquí podríamos agregar operadores matemáticos específicos como +, -, *, /, %
        alt((
            "**".value(Punctuation::Pow),
            "+".value(Punctuation::Plus),
            "-".value(Punctuation::Minus),
            "*".value(Punctuation::Multiply),
            "/".value(Punctuation::Div),
        ))
        .with_span()
        .map(|(op, span)| {
            Some(SpannedToken::new(
                Span::from(span),
                Token::Punctuation(op), // Por ahora usamos punctuation
            ))
        })
        .parse_next(input)
    }
}
