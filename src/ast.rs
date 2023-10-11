use crate::parser::common::Type;
use std::{borrow::Cow, str::FromStr};
use thiserror::Error;

pub use self::{fold::Folder, visitor::Visitor};

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
    pub members: Vec<ClassMember>,
}

#[derive(Debug)]
pub struct ClassMember {
    pub name: String,
    // FIXME: What visibility is when none is specified.
    pub visibility: Option<Visibility>,
    pub ty: Type,
    pub init_expr: Option<Box<Expression>>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub visibility: Visibility,
    pub arguments: Vec<Argument>,
    pub return_type: Type,
    pub block: Vec<Box<Statement>>,
}

#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub value_type: Type,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Variable {
        name: String,
        var_type: Type,
        value: Box<Expression>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    // A integer literal,
    Integer(i32),
    // A string literal.
    String(String),
    // A boolean,
    Bool(bool),
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

pub mod visitor {
    use super::*;

    pub trait Visitor<'ast>: Sized {
        type Error;

        fn visit_expression(&mut self, v: &'ast Expression) -> Result<(), Self::Error> {
            walk_expression(self, v)
        }

        fn visit_statement(&mut self, v: &'ast Statement) -> Result<(), Self::Error> {
            walk_statement(self, v)
        }

        fn visit_function(&mut self, v: &'ast Function) -> Result<(), Self::Error> {
            walk_function(self, v)
        }

        fn visit_class(&mut self, v: &'ast Class) -> Result<(), Self::Error> {
            walk_class(self, v)
        }

        fn visit_package(&mut self, v: &'ast Package) -> Result<(), Self::Error> {
            walk_package(self, v)
        }
    }

    pub fn walk_expression<'ast, E, V: Visitor<'ast, Error = E>>(
        visitor: &mut V,
        v: &'ast Expression,
    ) -> Result<(), E> {
        match v {
            Expression::BinaryOperation {
                lhs,
                operator: _,
                rhs,
            } => {
                visitor.visit_expression(&lhs)?;
                visitor.visit_expression(&rhs)?;
            }
            Expression::Integer(_) => {}
            Expression::Bool(_) => {}
            Expression::String(_) => {}
            Expression::Variable(_) => {}
        }

        Ok(())
    }

    pub fn walk_statement<'ast, E, V: Visitor<'ast, Error = E>>(
        visitor: &mut V,
        v: &'ast Statement,
    ) -> Result<(), E> {
        match v {
            Statement::Variable {
                name: _,
                var_type: _,
                value,
            } => visitor.visit_expression(&value)?,
        }

        Ok(())
    }

    pub fn walk_function<'ast, E, V: Visitor<'ast, Error = E>>(
        visitor: &mut V,
        v: &'ast Function,
    ) -> Result<(), E> {
        for v in &v.block {
            visitor.visit_statement(v)?;
        }

        Ok(())
    }

    pub fn walk_class<'ast, E, V: Visitor<'ast, Error = E>>(
        visitor: &mut V,
        v: &'ast Class,
    ) -> Result<(), E> {
        for v in &v.functions {
            visitor.visit_function(v)?;
        }

        Ok(())
    }

    pub fn walk_package<'ast, E, V: Visitor<'ast, Error = E>>(
        visitor: &mut V,
        v: &'ast Package,
    ) -> Result<(), E> {
        for v in &v.classes {
            visitor.visit_class(v)?;
        }

        Ok(())
    }
}

/// Abstract folder.
mod fold {
    use super::*;

    pub trait Folder {
        fn fold_expression(&mut self, v: Box<Expression>) -> Box<Expression> {
            v
        }

        fn fold_statement(&mut self, v: Box<Statement>) -> Box<Statement> {
            match *v {
                Statement::Variable {
                    name,
                    var_type,
                    value,
                } => Box::new(Statement::Variable {
                    name,
                    var_type,
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
                members,
            } = v;

            let functions = functions
                .into_iter()
                .map(|v| self.fold_function(v))
                .collect::<Vec<_>>();

            // FIXME: Fold all other things.

            Class {
                name,
                visibility,
                functions,
                members,
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
