//! Utilities for the grammar parser.

use crate::ast;

pub enum ClassPart {
    Member(()),
    Function(ast::Function),
}
