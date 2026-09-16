use winnow::{
    Parser,
    combinator::{alt, opt, peek, terminated},
    error::ParserError,
    token::any,
};

use crate::{
    error::{LexerErr, Result, Src},
    parsers::TokenParser,
    token::{Keyword, Span, SpannedToken, Token},
};

/// Matches a keyword literal, but only if it isn't immediately followed by
/// another identifier character (so `constructorLlamaSuper` isn't lexed as
/// `const` + `ructorLlamaSuper`).
fn keyword<'t>(literal: &'static str) -> impl Parser<Src<'t>, &'t str, LexerErr<'t>> {
    terminated(
        literal,
        peek(opt(any::<Src<'t>, LexerErr<'t>>)).verify(
            |next: &Option<char>| !matches!(next, Some(c) if c.is_alphanumeric() || *c == '_'),
        ),
    )
}

pub struct KeywordParser;

impl TokenParser for KeywordParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        let result = alt([
            keyword("if").value(Keyword::If),
            keyword("else").value(Keyword::Else),
            keyword("object").value(Keyword::Object),
            keyword("class").value(Keyword::Class),
            keyword("method").value(Keyword::Method),
            keyword("import").value(Keyword::Import),
            keyword("describe").value(Keyword::Describe),
            keyword("test").value(Keyword::Test),
            keyword("assert").value(Keyword::Assert),
            keyword("const").value(Keyword::Const),
            keyword("let").value(Keyword::Let),
            keyword("self").value(Keyword::This), // Using `self` as a keyword
            keyword("property").value(Keyword::Property),
            keyword("super").value(Keyword::Super),
            keyword("return").value(Keyword::Return),
            keyword("new").value(Keyword::New),
            keyword("inherits").value(Keyword::Inherits),
            keyword("override").value(Keyword::Override),
            keyword("fallible").value(Keyword::Fallible),
            keyword("try").value(Keyword::Try),
            keyword("abstract").value(Keyword::Abstract),
            keyword("mixin").value(Keyword::Mixin),
            keyword("with").value(Keyword::With),
            keyword("catch").value(Keyword::Catch),
        ])
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

    #[test]
    fn test_identifier_prefixed_by_keyword() {
        let mut input = Src::new("constructorLlamaSuper");
        let result = KeywordParser::parse(&mut input);

        assert!(result.is_err());
    }
}
