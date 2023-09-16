use logos::Logos;
use std::fmt;

pub enum Token {
    #[token("var")]
    KeywordVar,
    #[token("package")]
    KeywordPackage,
    #[token("class")]
    KeywordClass,

    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().parse())]
    Identifier(String),
    #[regex("\d+", |lex| lex.slice().parse())]
    Integer(i32),

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,

    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSub,
    #[token("*")]
    OperatorMul,
    #[token("/")]
    OperatorDiv,

    // Slash comments.
    #[regex(r"\/\/.*\n?", logos::skip)]
    #[error]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}