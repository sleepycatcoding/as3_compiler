use logos::{Logos, SpannedIter};

mod token;

pub use token::{abs, as3};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexicalError {
    InvalidToken { line: usize, column: usize },
}

pub struct Lexer<'input, T: Logos<'input, Source = str, Extras = (usize, usize)>> {
    token_stream: SpannedIter<'input, T>,
}

impl<'input, T: Logos<'input, Source = str, Extras = (usize, usize)>> Lexer<'input, T> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: T::lexer(input).spanned(),
        }
    }
}

impl<'input, T: Logos<'input, Source = str, Extras = (usize, usize)>> Iterator
    for Lexer<'input, T>
{
    type Item = Spanned<T, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| match token {
            Err(_) => Err(LexicalError::InvalidToken {
                line: self.token_stream.extras.0,
                column: span.start - self.token_stream.extras.1,
            }),
            Ok(token) => Ok((span.start, token, span.end)),
        })
    }
}
