//! Utilities for the grammar parser.

use crate::ast;

pub enum ClassPart {
    Member(ast::ClassMember),
    Function(ast::Function),
}
