use std::{borrow::Cow, str::FromStr};
use thiserror::Error;

pub use fold::Folder;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown visibility: {0}")]
    UnknownVisibility(String),
    // FIXME: Include expression name here.
    #[error("Cannot convert 'Unimplemented' to string")]
    CannotConvertToString,
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
    pub block: Vec<Box<Statement>>,
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
    // A integer literal,
    Integer(i32),
    // A string literal.
    String(String),
    // A variable.
    Variable(String),
    // A binary operation.
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

impl<'a> TryFrom<&'a Expression> for Cow<'a, String> {
    type Error = Error;

    fn try_from(value: &'a Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::Integer(v) => Ok(Cow::Owned(v.to_string())),
            Expression::String(v) => Ok(Cow::Borrowed(v)),
            _ => Err(Error::CannotConvertToString),
        }
    }
}

// impl TryInto<String> for Expression {
//     fn to_string(&self) -> String {
//         match self {
//             // FIXME: Sad clone face.
//             Expression::String(v) => v.clone(),
//             Expression::Integer(v) => v.to_string(),
//             _ => panic!("Cannot convert {:?} to a string", self),
//         }
//     }
// }

/// Abstract folder.
mod fold {
    use super::*;

    pub trait Folder {
        fn fold_name(&mut self, v: String) -> String {
            v
        }

        fn fold_expression(&mut self, v: Box<Expression>) -> Box<Expression> {
            if let Expression::Variable(name) = *v {
                Box::new(Expression::Variable(self.fold_name(name)))
            } else {
                v
            }
        }

        fn fold_statement(&mut self, v: Box<Statement>) -> Box<Statement> {
            match *v {
                Statement::Variable { name, value } => Box::new(Statement::Variable {
                    name: self.fold_name(name),
                    value: self.fold_expression(value),
                }),
            }
        }

        fn fold_function(&mut self, v: Function) -> Function {
            let Function {
                name,
                visibility,
                arguments,
                return_type,
                block,
            } = v;

            let name = self.fold_name(name);

            let block = block
                .into_iter()
                .map(|v| self.fold_statement(v))
                .collect::<Vec<_>>();

            // FIXME: Fold all other things.

            Function {
                name,
                visibility,
                arguments,
                return_type,
                block,
            }
        }

        fn fold_class(&mut self, v: Class) -> Class {
            let Class {
                name,
                visibility,
                functions,
            } = v;

            let name = self.fold_name(name);
            let functions = functions
                .into_iter()
                .map(|v| self.fold_function(v))
                .collect::<Vec<_>>();

            // FIXME: Fold all other things.

            Class {
                name,
                visibility,
                functions,
            }
        }

        fn fold_package(&mut self, v: Package) -> Package {
            let Package { name, classes } = v;

            let classes = classes
                .into_iter()
                .map(|v| self.fold_class(v))
                .collect::<Vec<_>>();

            // FIXME: Fold all other things.

            Package { name, classes }
        }
    }
}
