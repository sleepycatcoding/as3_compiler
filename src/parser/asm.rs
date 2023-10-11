use crate::parser::common::Type;
use swf::avm2::types::Op;

pub mod grammar;

/// Represents an ActionScript assembly opcode.
///
/// This is a step above, a normal opcode, since
/// assembly has features to make writing it easier.
#[derive(Debug)]
pub enum AssemblyOp {
    Raw(Op),
    Label(String),
    IfTrue(String),
    IfFalse(String),
    IfEq(String),
    PushString(String),
}

impl From<Op> for AssemblyOp {
    fn from(value: Op) -> Self {
        Self::Raw(value)
    }
}

#[derive(Debug)]
pub struct Function {
    name: String,
    args: Vec<Type>,
    ops: Vec<AssemblyOp>,
}
