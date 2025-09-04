use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Punctuation, Span, SpannedToken, Token},
};
use winnow::{Parser, combinator::alt};

pub struct OperatorParser;

impl TokenParser for OperatorParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        // Por ahora los operadores están en punctuation
        // Aquí podríamos agregar operadores matemáticos específicos como +, -, *, /, %
        alt((
            "*".value(Punctuation::Multiply),
            "+".value(Punctuation::Plus),
            "-".value(Punctuation::Minus),
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
