use logos::{Logos, SpannedIter};

mod token;

pub use token::{as3, asm};

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
            Ok(token) => Ok((
                self.token_stream.extras.0,
                token,
                span.start - self.token_stream.extras.1,
            )),
        })
    }
}

use chumsky::prelude::*;

pub type Span = SimpleSpan<usize>;

/// A lexer token.
#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src> {
    Null,
    Undefined,
    Bool(bool),
    Int(i32),
    Str(&'src str),
    Op(&'src str),
    /// Control characters.
    Ctrl(char),
    Ident(&'src str),
    Package,
    Class,
    Fn,
    Var,
    If,
    Else,
}

impl<'src> std::fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Undefined => write!(f, "undefined"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Int(x) => write!(f, "{}", x),
            Self::Str(x) => write!(f, "{}", x),
            Self::Op(x) => write!(f, "{}", x),
            Self::Ctrl(x) => write!(f, "{}", x),
            Self::Ident(x) => write!(f, "{}", x),
            Self::Package => write!(f, "package"),
            Self::Class => write!(f, "class"),
            Self::Fn => write!(f, "function"),
            Self::Var => write!(f, "var"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
        }
    }
}

pub fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Rich<'src, char, Span>>> {
    // Int parser.
    // FIXME: negative values.
    let int = text::int(10)
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Int);

    // String parser.
    let str_ = just('"')
        .ignore_then(none_of('"').repeated())
        .then_ignore(just('"'))
        .to_slice()
        .map(Token::Str);

    // Operator parser.
    let op = one_of("+*-/!=")
        .repeated()
        .at_least(1)
        .to_slice()
        .map(Token::Op);

    // Control character parser.
    let ctrl = one_of("()[]{}:;,").map(Token::Ctrl);

    // Keyword and identifier parser.
    let ident = text::ascii::ident().map(|ident: &str| match ident {
        "package" => Token::Package,
        "class" => Token::Class,
        "function" => Token::Fn,
        "var" => Token::Var,
        "if" => Token::If,
        "else" => Token::Else,
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "null" => Token::Null,
        "undefined" => Token::Undefined,
        _ => Token::Ident(ident),
    });

    // Combined parser
    let token = int.or(str_).or(op).or(ctrl).or(ident);

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}
