use crate::ast::Visibility;
use logos::Logos;
use std::fmt;

#[derive(Clone, Debug, Logos)]
// Slash comments.
#[logos(skip r"//.*\n?")]
#[logos(skip " ")]
#[logos(skip "\n")]
pub enum Token {
    #[token("var")]
    KeywordVar,
    #[token("package")]
    KeywordPackage,
    #[token("class")]
    KeywordClass,
    #[token("function")]
    KeywordFunction,

    #[regex("(public|protected|private)", callback = |lex| lex.slice().parse().ok())]
    KeywordVisibility(Visibility),

    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", priority = 2, callback = |lex| lex.slice().parse().ok())]
    Identifier(String),
    #[regex("\\d+", |lex| lex.slice().parse().ok())]
    Integer(i32),

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LCurlyBracket,
    #[token("}")]
    RCurlyBracket,
    #[token("=")]
    Assign,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
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
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
