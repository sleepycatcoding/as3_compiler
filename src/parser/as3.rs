//! ActionScript 3 grammar parser.

use crate::ast;

pub mod grammar;

pub enum ClassPart {
    Member(ast::ClassMember),
    Function(ast::Function),
}
