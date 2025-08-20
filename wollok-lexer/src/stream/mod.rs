use std::collections::VecDeque;

use crate::{
    error::{Result, Src},
    parsers::{
        CommentParser, IdentifierParser, KeywordParser, LiteralParser, OperatorParser,
        PunctuationParser, TokenParser, WhitespaceParser,
    },
    token::SpannedToken,
};

use winnow::{Parser, combinator::alt, error::ParserError};

pub struct TokenStream<'t> {
    input: Src<'t>,
    finished: bool,
}

impl<'t> TokenStream<'t> {
    #[must_use]
    pub fn new(input: &'t str) -> Self {
        Self {
            input: Src::new(input),
            finished: false,
        }
    }

    /// Intenta parsear el siguiente token
    fn next_token(&mut self) -> Result<'t, Option<SpannedToken>> {
        if self.finished {
            return Ok(None);
        }

        if self.input.is_empty() {
            self.finished = true;
            return Ok(None);
        }

        if let Ok(None) = WhitespaceParser::parse(&mut self.input) {
            return self.next_token();
        }

        let result = alt((
            CommentParser::parse,     // Comentarios
            KeywordParser::parse,     // Keywords antes que identifiers
            LiteralParser::parse,     // Literales (números, strings, booleans)
            IdentifierParser::parse,  // Identificadores
            OperatorParser::parse,    // Operadores
            PunctuationParser::parse, // Puntuación
        ))
        .parse_next(&mut self.input);

        match result {
            Ok(Some(token)) => Ok(Some(token)),
            Ok(None) => {
                // Ningún parser pudo manejar el input, esto es un error
                self.finished = true;
                Err(crate::error::LexerErr::from_input(&self.input))
            }
            Err(e) => {
                self.finished = true;
                Err(e)
            }
        }
    }

    /// Recolecta todos los tokens restantes en un Vec
    ///
    /// # Errors
    /// Retorna un error si ocurre un error de lexer al parsear los tokens.
    pub fn collect_all(mut self) -> Result<'t, Vec<SpannedToken>> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next()? {
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Recolecta todos los tokens restantes en un `VecDeque` (para compatibilidad)
    ///
    /// # Errors
    /// Retorna un error si ocurre un error de lexer al parsear los tokens.
    pub fn collect_deque(mut self) -> Result<'t, VecDeque<SpannedToken>> {
        let mut tokens = VecDeque::new();

        while let Some(token) = self.next()? {
            tokens.push_back(token);
        }

        Ok(tokens)
    }
}

impl<'t> Iterator for TokenStream<'t> {
    type Item = Result<'t, SpannedToken>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

// Implementación de next() que retorna Result<Option<SpannedToken>>
impl<'t> TokenStream<'t> {
    /// Versión de `next()` que retorna Result en lugar de Option<Result>
    fn next(&mut self) -> Result<'t, Option<SpannedToken>> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{T, ident, kw, lit};

    use super::*;

    #[test]
    fn test_class_example() {
        let source = "class Animal { property energy = 10 }";
        let stream = TokenStream::new(source);
        let tokens = stream.collect_all().unwrap();

        assert_eq!(
            tokens,
            vec![
                kw!(Class),
                ident!("Animal"),
                T!(OpenBrace),
                kw!(Property),
                ident!("energy"),
                T!(Equals),
                lit!(10),
                T!(CloseBrace)
            ]
        );
    }

    #[test]
    fn test_self_example() {
        let source = "object dummy { method run() = self }";
        let stream = TokenStream::new(source);
        let tokens = stream.collect_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                kw!(Object),
                ident!("dummy"),
                T!(OpenBrace),
                kw!(Method),
                ident!("run"),
                T!(OpenParen),
                T!(CloseParen),
                T!(Equals),
                kw!(This),
                T!(CloseBrace)
            ]
        );
    }

    #[test]
    fn test_code_example() {
        let source = "object dummy {\n\tconst age = 42 }";
        let stream = TokenStream::new(source);
        let tokens = stream.collect_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                kw!(Object),
                ident!("dummy"),
                T!(OpenBrace),
                T!(Newline),
                kw!(Const),
                ident!("age"),
                T!(Equals),
                lit!(42),
                T!(CloseBrace)
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let mut stream = TokenStream::new("");
        assert!(stream.next().unwrap().is_none());
    }

    #[test]
    fn test_simple_tokens() {
        let mut stream = TokenStream::new("42 true");

        // Primer token debería ser el número
        let Ok(Some(token)) = stream.next() else {
            panic!("Expected a token");
        };

        assert_eq!(token.token, lit!(42));

        let Ok(Some(token)) = stream.next() else {
            panic!("Expected a token");
        };

        assert_eq!(token.token, lit!(true));
    }

    #[test]
    fn test_collect_all() {
        let stream = TokenStream::new("42 + 3");
        let tokens = stream.collect_all().unwrap();

        assert_eq!(tokens, vec![lit!(42), T!(Plus), lit!(3)]);
    }
}
