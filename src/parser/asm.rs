use crate::parser::common::Type;
use swf::avm2::types::Op;

pub mod grammar;

/// Represents a Id source, which can be anything (e.g. .function_id).
#[derive(Debug)]
pub enum IdSource {
    Function(String),
    Other(String),
}

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
    PushNamespace(String),
    FindProperty(String),
    FindPropStrict(String),
    CallProperty(String, u32),
    CallPropVoid(String, u32),
    Call(u32),
    NewFunction(IdSource),
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
