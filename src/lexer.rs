use logos::{Logos, SpannedIter};

mod token;

pub use token::Token;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexicalError {
    InvalidToken { line: usize, column: usize },
}

pub struct Lexer<'input> {
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

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
