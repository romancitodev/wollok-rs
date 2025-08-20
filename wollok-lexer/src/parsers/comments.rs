use winnow::{Parser, combinator::preceded, token::take_while};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Span, SpannedToken, Token},
};

pub struct CommentParser;

impl TokenParser for CommentParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        let (content, span) = preceded("//", take_while(0.., |c: char| c != '\n' && c != '\r'))
            .with_span()
            .parse_next(input)?;

        Ok(Some(SpannedToken::new(
            Span::from(span),
            Token::Comment(content.to_string()),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cmt, error::Src};

    #[test]
    fn test_single_line_comment() {
        let mut input = Src::new("// esto es un comentario");
        let result = CommentParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, cmt!(" esto es un comentario"));
    }

    #[test]
    fn test_comment_with_newline() {
        let mut input = Src::new("// comentario\nmas_codigo");
        let result = CommentParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, cmt!(" comentario"));
    }

    #[test]
    fn test_not_a_comment() {
        let mut input = Src::new("/ not a comment");
        let result = CommentParser::parse(&mut input);

        assert!(result.is_err());
    }
}
