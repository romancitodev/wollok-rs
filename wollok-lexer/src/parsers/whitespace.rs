use winnow::{Parser, token::take_while};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::SpannedToken,
};

pub struct WhitespaceParser;

impl TokenParser for WhitespaceParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        _ = take_while(1.., |c: char| c == ' ' || c == '\t').parse_next(input)?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Src;

    #[test]
    fn test_spaces() {
        let mut input = Src::new("   ");
        let result = WhitespaceParser::parse(&mut input).unwrap();

        assert!(result.is_none()); // Whitespace debe ser ignorado
    }

    #[test]
    fn test_tabs() {
        let mut input = Src::new("\t\t");
        let result = WhitespaceParser::parse(&mut input).unwrap();

        assert!(result.is_none()); // Whitespace debe ser ignorado
    }

    #[test]
    fn test_mixed_whitespace() {
        let mut input = Src::new(" \t ");
        let result = WhitespaceParser::parse(&mut input).unwrap();

        assert!(result.is_none()); // Whitespace debe ser ignorado
    }

    #[test]
    fn test_not_whitespace() {
        let mut input = Src::new("abc");
        let result = WhitespaceParser::parse(&mut input);

        assert!(result.is_err());
    }
}
