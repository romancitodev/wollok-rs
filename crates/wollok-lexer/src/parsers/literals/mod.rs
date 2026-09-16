use winnow::{Parser, combinator::alt};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Literal, Span, SpannedToken, Token},
};

pub mod booleans;
pub mod numbers;
pub mod strings;

pub use booleans::BooleanParser;
pub use numbers::NumberParser;
pub use strings::StringParser;

pub struct LiteralParser;

impl TokenParser for LiteralParser {
    fn parse<'err>(input: &mut Src<'err>) -> Result<'err, Option<SpannedToken>> {
        alt((
            StringParser::parse,
            NumberParser::parse,
            BooleanParser::parse,
            "null".with_span().map(|(_null, span)| {
                Some(SpannedToken::new(
                    Span::from(span),
                    Token::Literal(Literal::Null),
                ))
            }),
        ))
        .parse_next(input)
    }
}
