use {
    crate::token::Span,
    ariadne::{Label, Report, ReportKind, Source},
    std::{backtrace::Backtrace, fmt},
    winnow::{
        LocatingSlice,
        error::{AddContext, ParserError},
        stream::{Location, Stream},
    },
};

pub type Src<'source> = LocatingSlice<&'source str>;
pub type Result<'err, T = ()> = std::result::Result<T, LexerErr<'err>>;

#[derive(Debug)]
pub struct LexerErr<'e> {
    base: &'e str,
    span: Span,
    labels: Vec<(Span, String)>,
    backtrace: Backtrace,
}

pub trait LexerExt<'lex> {
    type Base;
    type Span;
    fn base(&self) -> Self::Base;
    fn span(&self) -> Self::Span;
    fn error(&self, msg: impl fmt::Display) -> !;
}

impl<'a> LexerExt<'a> for Src<'a> {
    type Base = &'a str;
    type Span = Span;

    fn base(&self) -> Self::Base {
        let mut base = *self;
        base.reset_to_start();
        *base
    }

    fn span(&self) -> Self::Span {
        Span::char(self.current_token_start())
    }

    fn error(&self, msg: impl fmt::Display) -> ! {
        _ = Report::build(ariadne::ReportKind::Error, self.span())
            .with_code(3)
            .with_label(
                Label::new(self.span())
                    .with_message(msg.to_string())
                    .with_color(ariadne::Color::BrightRed),
            )
            .finish()
            .eprint(Source::from(self.base()));

        std::process::exit(1);
    }
}

impl<'a> ParserError<Src<'a>> for LexerErr<'a> {
    type Inner = Self;

    fn into_inner(self) -> winnow::Result<Self::Inner, Self> {
        Ok(self)
    }

    fn from_input(input: &Src<'a>) -> Self {
        Self {
            base: input.base(),
            span: input.span(),
            labels: Vec::new(),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl<'i, C: ToString> AddContext<Src<'i>, C> for LexerErr<'i> {
    fn add_context(
        mut self,
        input: &Src<'i>,
        _token_start: &<Src<'i> as Stream>::Checkpoint,
        context: C,
    ) -> Self {
        self.labels.push((input.span(), context.to_string()));
        self
    }
}

impl fmt::Display for LexerErr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        _ = Report::build(ReportKind::Error, self.span)
            .with_labels(
                self.labels
                    .iter()
                    .map(|(span, msg)| Label::new(*span).with_message(msg)),
            )
            .finish()
            .eprint(Source::from(self.base));

        let backtrace = self.backtrace.to_string();

        let mut backtrace = backtrace.split('\n');

        writeln!(f, "Backtrace:")?;

        while let Some(line) = backtrace.next() {
            let Some(colon_idx) = line.find(':') else {
                break;
            };

            let module = &line[colon_idx + 2..];
            let Some(file_line) = backtrace.next() else {
                break;
            };

            if module.starts_with("std") || module.starts_with("core") {
                continue;
            }

            let Some(at_idx) = file_line.find("at ") else {
                break;
            };

            let file = &file_line[at_idx + 3..];

            if !file.starts_with('.') {
                continue;
            }

            writeln!(f, "  \x1b[31mat \x1b[33m{file} \x1b[31m({module})\x1b[0m")?;
        }

        Ok(())
    }
}
