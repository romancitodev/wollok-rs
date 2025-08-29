use winnow::{Parser, combinator::alt, error::ParserError};

use crate::{
    error::{LexerErr, Result, Src},
    parsers::TokenParser,
    token::{Keyword, Span, SpannedToken, Token},
};

pub struct KeywordParser;

impl TokenParser for KeywordParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        let result = alt((
            "if".value(Keyword::If),
            "else".value(Keyword::Else),
            "object".value(Keyword::Object),
            "class".value(Keyword::Class),
            "method".value(Keyword::Method),
            "import".value(Keyword::Import),
            "describe".value(Keyword::Describe),
            "test".value(Keyword::Test),
            "assert".value(Keyword::Assert),
            "const".value(Keyword::Const),
            "let".value(Keyword::Let),
            "self".value(Keyword::This), // Using `self` as a keyword
            "property".value(Keyword::Property),
            "super".value(Keyword::Super),
            "return".value(Keyword::Return),
            "new".value(Keyword::New),
        ))
        .with_span()
        .map(|(keyword, span)| Some(SpannedToken::new(Span::from(span), Token::Keyword(keyword))))
        .parse_next(input)?;

        match result {
            Some(token) => Ok(Some(token)),
            None => Err(LexerErr::from_input(input)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Src;

    #[test]
    fn test_if_keyword() {
        let mut input = Src::new("if");
        let result = KeywordParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        if let Token::Keyword(Keyword::If) = &token.token {
            // success
        } else {
            panic!("Expected If keyword");
        }
    }

    #[test]
    fn test_object_keyword() {
        let mut input = Src::new("object");
        let result = KeywordParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        if let Token::Keyword(Keyword::Object) = &token.token {
            // success
        } else {
            panic!("Expected Object keyword");
        }
    }

    #[test]
    fn test_not_a_keyword() {
        let mut input = Src::new("variable");
        let result = KeywordParser::parse(&mut input);

        assert!(result.is_err());
    }
}
