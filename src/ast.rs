use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown visibility: {0}")]
    UnknownVisibility(String),
}

#[derive(Debug)]
pub struct Package {
    pub name: Option<String>,
    pub classes: Vec<Class>,
}

#[derive(Clone, Debug)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub visibility: Visibility,
    pub functions: Vec<Function>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub visibility: Visibility,
    pub arguments: Vec<Argument>,
    pub return_type: Option<String>,
    pub statements: Vec<Box<Statement>>,
}

#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub value_type: Option<String>,
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
