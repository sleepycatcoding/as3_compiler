use std::{convert::Infallible, str::FromStr};

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Any,
    Void,
    Int,
    Uint,
    Bool,
    Other(String),
}

impl FromStr for Type {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "*" => Ok(Type::Any),
            "void" => Ok(Type::Void),
            "int" => Ok(Type::Int),
            "uint" => Ok(Type::Uint),
            "Boolean" => Ok(Type::Bool),
            s => Ok(Type::Other(s.to_owned())),
        }
    }
}
