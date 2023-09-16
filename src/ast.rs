use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown visibility: {0}")]
    UnknownVisibility(String),
}

#[derive(Debug)]
pub struct File {
    pub package: Option<String>,
}

#[derive(Debug)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl FromStr for Visibility {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(Visibility::Public),
            "protected" => Ok(Visibility::Protected),
            "private" => Ok(Visibility::Private),
            s => Err(Error::UnknownVisibility(s.to_owned())),
        }
    }
}

#[derive(Debug)]
pub struct Class {
    name: String,
    visibility: Visibility,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Variable {
        name: String,
        value: Box<Expression>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Integer(i32),
    Variable(String),
    BinaryOperation {
        lhs: Box<Expression>,
        operator: Operator,
        rhs: Box<Expression>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}
