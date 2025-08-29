use wollok_lexer::{
    lexer::TokenStream,
    token::{Span, SpannedToken, Token},
    macros::T,
};

use ariadne::{Color, Label, Report, ReportBuilder, ReportKind, Source};
use std::{collections::VecDeque, fmt};

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    pub base: &'a str,
    pub last_offset: usize,
    pub tokens: VecDeque<SpannedToken>,
}

#[derive(Debug)]
pub struct PeekedToken<'a, 's> {
    pub src: &'s mut Ast<'a>,
    pub token: SpannedToken,
    pub last_offset: usize,
}

impl<'i> Ast<'i> {
    #[must_use]
    pub fn new(base: &'i str, tokens: TokenStream<'i>) -> Self {
        Ast {
            base,
            last_offset: 0,
            tokens: tokens.collect_deque().unwrap_or_else(|_| VecDeque::new()),
        }
    }

    // ======== New Token API - Phase 1 ========
    
    /// Check if next token matches without consuming it
    pub fn check(&mut self, expected: &Token) -> bool {
        if let Some(peeked) = self.peek() {
            let matches = *peeked == *expected;
            peeked.recover();
            matches
        } else {
            false
        }
    }

    /// Consume token if it matches, return whether it was consumed
    pub fn consume(&mut self, expected: &Token) -> bool {
        if let Some(peeked) = self.peek() {
            if *peeked == *expected {
                let _ = peeked.accept();
                true
            } else {
                peeked.recover();
                false
            }
        } else {
            false
        }
    }

    /// Move to next token without any checks
    pub fn advance(&mut self) -> Option<SpannedToken> {
        self.tokens.pop_front().inspect(|t| self.last_offset = t.span.to)
    }

    /// Look at next token without consuming it (simple version)
    pub fn peek_token(&mut self) -> Option<Token> {
        if let Some(peeked) = self.peek() {
            let token = peeked.token.token.clone();
            peeked.recover();
            Some(token)
        } else {
            None
        }
    }

    pub fn peek<'a>(&'a mut self) -> Option<PeekedToken<'i, 'a>> {
        self.tokens.pop_front().map(|token| {
            let last_span = self.last_offset;
            self.last_offset = token.span.to;
            PeekedToken {
                token,
                last_offset: last_span,
                src: self,
            }
        })
    }

    pub fn peek_expect<'a>(&'a mut self) -> PeekedToken<'i, 'a> {
        // The implementation cannot be done with `peek` call
        // because of borrow checker :|
        let Some(token) = self.tokens.pop_front() else {
            self.error_in_place("Unexpected EOF");
        };

        let last_offset = self.last_offset;
        self.last_offset = token.span.to;
        PeekedToken {
            token,
            last_offset,
            src: self,
        }
    }

    pub fn expect(&mut self) -> SpannedToken {
        self.tokens
            .pop_front()
            .inspect(|t| self.last_offset = t.span.to)
            .unwrap_or_else(|| self.error_in_place("Unexpected EOF"))
    }

    pub fn expect_msg(&mut self, msg: impl fmt::Display) -> SpannedToken {
        self.tokens
            .pop_front()
            .inspect(|t| self.last_offset = t.span.to)
            .unwrap_or_else(|| self.error_in_place(format!("Unexpected EOF. {msg}")))
    }

    pub fn expect_match<T>(
        &mut self,
        msg: impl fmt::Display + Clone,
        predicate: impl Fn(SpannedToken) -> Option<T>,
    ) -> T {
        let expected_err = format!("Expected {msg}");
        let first = self.expect_msg(&expected_err);

        let span = first.span;
        let unexpected_err = format!("Unexpected token: {first:?}");

        if let Some(t) = predicate(first) {
            self.last_offset = span.to;
            t
        } else {
            self.error_build(
                span,
                |b| {
                    b.with_message(unexpected_err).with_label(
                        Label::new(span)
                            .with_color(Color::BrightRed)
                            .with_message(expected_err),
                    )
                },
                msg.clone(),
            );
        }
    }

    pub fn expect_token(&mut self, token: &Token) -> SpannedToken {
        let mut checkpoint = self.clone();
        let fetched = self.peek_expect().token;
        checkpoint.expect_match(format!("{token:#?} but got {fetched:#?}"), |t| {
            (*t == *token).then_some(t)
        })
    }

    pub fn error_in_place(&self, msg: impl fmt::Display + Clone) -> ! {
        let span = Span::char(self.last_offset - 1);
        self.error_build(
            span,
            |b| {
                b.with_message(msg.clone()).with_label(
                    Label::new(span)
                        .with_message(msg.clone())
                        .with_color(ariadne::Color::BrightRed),
                )
            },
            msg.clone(),
        )
    }

    pub fn error_at(&self, span: Span, msg: impl fmt::Display + Clone) -> ! {
        let cloned_msg = msg.clone();
        self.error_build(
            span,
            |b| {
                b.with_message(&msg).with_label(
                    Label::new(span)
                        .with_message(msg)
                        .with_color(ariadne::Color::BrightRed),
                )
            },
            cloned_msg,
        )
    }

    /// # Panics
    /// Only when it's called
    pub fn error_build(
        &self,
        span: Span,
        fun: impl FnOnce(ReportBuilder<Span>) -> ReportBuilder<Span>,
        msg: impl fmt::Display,
    ) -> ! {
        if !cfg!(test) {
            _ = fun(Report::build(ReportKind::Error, span))
                .finish()
                .eprint(Source::from(self.base));
        }

        panic!("{msg}");
    }

    // ======== Helper Methods - Phase 1 ========

    /// Try parsing with automatic rollback on failure
    pub fn optional<T>(&mut self, mut parser: impl FnMut(&mut Self) -> Option<T>) -> Option<T> {
        // Create a checkpoint by saving the current state
        let checkpoint_tokens = self.tokens.clone();
        let checkpoint_offset = self.last_offset;
        
        // Try to parse
        if let Some(result) = parser(self) {
            Some(result)
        } else {
            // Rollback on failure
            self.tokens = checkpoint_tokens;
            self.last_offset = checkpoint_offset;
            None
        }
    }

    /// Parse a comma-separated list with generic element parser
    pub fn parse_separated_list<T>(
        &mut self,
        element_parser: impl Fn(&mut Self) -> T,
        separator: &Token,
        terminator: &Token,
    ) -> Vec<T> {
        let mut elements = Vec::new();

        // Check for empty list
        if self.check(terminator) {
            self.consume(terminator);
            return elements;
        }

        // Parse first element
        elements.push(element_parser(self));

        // Parse remaining elements
        while self.consume(separator) {
            // Check for trailing separator
            if self.check(terminator) {
                self.consume(terminator);
                break;
            }
            elements.push(element_parser(self));
        }

        // Consume terminator
        if !self.consume(terminator) {
            self.error_in_place(format!("Expected '{}'", terminator));
        }

        elements
    }

    /// Parse a comma-separated list of identifiers (for method parameters)
    pub fn parse_identifier_list(&mut self, terminator: &Token) -> Vec<crate::item::Ident> {
        self.parse_separated_list(
            |parser| {
                let name = parser.expect_match("Expected identifier", |t| t.into_ident());
                crate::item::Ident { name }
            },
            &T!(Comma),
            terminator,
        )
    }

    /// Unified whitespace and comment handling
    pub fn skip_trivia(&mut self) {
        while let Some(token) = self.peek_token() {
            match token {
                Token::Comment(_) => {
                    let _ = self.advance();
                },
                T!(Newline) => {
                    let _ = self.advance();
                },
                _ => break,
            }
        }
    }
}

impl PeekedToken<'_, '_> {
    #[must_use]
    pub fn accept(self) -> SpannedToken {
        self.token
    }

    pub fn recover(self) {
        self.src.last_offset = self.last_offset;
        self.src.tokens.push_front(self.token);
    }
}

impl std::ops::Deref for PeekedToken<'_, '_> {
    type Target = SpannedToken;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}
