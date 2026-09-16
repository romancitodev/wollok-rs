use winnow::{
    Parser,
    combinator::opt,
    token::{one_of, take_while},
};

use crate::{
    error::{Result, Src},
    parsers::TokenParser,
    token::{Literal, Span, SpannedToken, Token},
};

pub struct NumberParser;

impl TokenParser for NumberParser {
    fn parse<'t>(input: &mut Src<'t>) -> Result<'t, Option<SpannedToken>> {
        // Parseamos enteros y flotantes
        (
            opt(one_of(['+', '-'])), // signo opcional
            take_while(1.., |c: char| c.is_ascii_digit()).map(|s: &str| s), // parte entera
            opt((
                '.',
                take_while(0.., |c: char| c.is_ascii_digit()).map(|s: &str| s),
            )),
        )
            .with_span()
            .map(|((sign, number, fract), span)| {
                let literal = if let Some(('.', fract)) = fract {
                    match format!("{number}.{fract}").parse::<f64>() {
                        Ok(f) => Some(Literal::Float(f)),
                        Err(_) => return None,
                    }
                } else {
                    match number.parse::<i64>() {
                        Ok(i) => Some(Literal::Integer(i)),
                        Err(_) => return None,
                    }
                };

                let lit = literal?;

                let lit = match (sign, lit) {
                    (Some('-'), Literal::Integer(i)) => Literal::Integer(-i),
                    (Some('-'), Literal::Float(f)) => Literal::Float(-f),
                    (_, l) => l, // Para None o '+', mantener el valor original
                };
                Some(SpannedToken::new(Span::from(span), Token::Literal(lit)))
            })
            .parse_next(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Src, lit};

    #[test]
    fn test_positive_integer() {
        let mut input = Src::new("42");
        let result = NumberParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!(42));
    }

    #[test]
    fn test_negative_integer() {
        let mut input = Src::new("-15");
        let result = NumberParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!(-15));
    }

    #[test]
    fn test_float() {
        let binding = std::f64::consts::PI.to_string();
        let mut input = Src::new(&binding);
        let result = NumberParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!(std::f64::consts::PI));
    }

    #[test]
    fn test_negative_float() {
        let mut input = Src::new("-2.5");
        let result = NumberParser::parse(&mut input).unwrap();

        assert!(result.is_some());
        let token = result.unwrap();

        assert_eq!(token, lit!(-2.5));
    }
}
