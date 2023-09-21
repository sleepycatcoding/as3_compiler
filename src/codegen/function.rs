//! The purpose of this module is to generate ABC bytecode for functions.

use swf::avm2::types::Op;

use crate::ast::{
    visitor::{walk_function, walk_statement},
    Expression, Operator, Statement, Type, Visitor,
};
use std::collections::HashMap;

use super::{CodeGenerationContext, WrappedOp};

/// A visitor used to collect and assign indices to all variables/arguments referenced in a function.
#[derive(Debug)]
pub struct VariableCollectVisitor<'a> {
    current_index: u32,
    // FIXME: Maybe store type here too since it might be needed later?
    variables: HashMap<&'a String, u32>,
}

impl Default for VariableCollectVisitor<'_> {
    fn default() -> Self {
        Self {
            current_index: 0,
            variables: HashMap::new(),
        }
    }
}

impl<'a> VariableCollectVisitor<'a> {
    /// Generates a new unique index.
    fn fetch_index(&mut self) -> u32 {
        self.current_index += 1;
        self.current_index
    }

    /// Adds a variable to contained variables.
    fn add_var(&mut self, name: &'a String) {
        if !self.variables.contains_key(name) {
            let index = self.fetch_index();
            self.variables.insert(name, index);
        }
    }
}

impl<'ast> Visitor<'ast> for VariableCollectVisitor<'ast> {
    type Error = ();

    fn visit_statement(&mut self, v: &'ast Statement) -> Result<(), Self::Error> {
        match v {
            Statement::Variable {
                name,
                var_type: _,
                value: _,
            } => self.add_var(&name),
        }

        walk_statement(self, v)
    }

    fn visit_function(&mut self, v: &'ast crate::ast::Function) -> Result<(), Self::Error> {
        // NOTE: First argument is always self/this argument, but index 0 is never given out so this is fine.

        // FIXME: What if "arguments" is referenced and stuff like that?
        //        Also variadic arguments.
        // Some locals are reserved for arguments.
        for arg in &v.arguments {
            self.add_var(&arg.name)
        }

        // Walk the rest of the function.
        walk_function(self, v)
    }

    fn visit_class(&mut self, _v: &'ast crate::ast::Class) -> Result<(), Self::Error> {
        panic!("Unsupported operation")
    }

    fn visit_package(&mut self, _v: &'ast crate::ast::Package) -> Result<(), Self::Error> {
        panic!("Unsupported operation")
    }
}

#[derive(Debug)]
pub struct FunctionGenerator<'ast, 'a> {
    context: &'a mut CodeGenerationContext<'ast>,
    variables: HashMap<&'ast String, u32>,
}

impl<'ast, 'a> FunctionGenerator<'ast, 'a> {
    pub fn new(
        variables: VariableCollectVisitor<'ast>,
        context: &'a mut CodeGenerationContext<'ast>,
    ) -> Self {
        Self {
            context,
            variables: variables.variables,
        }
    }
}

impl<'ast, 'a> Visitor<'ast> for FunctionGenerator<'ast, 'a> {
    type Error = ();

    fn visit_expression(&mut self, v: &'ast crate::ast::Expression) -> Result<(), Self::Error> {
        match v {
            Expression::BinaryOperation { lhs, operator, rhs } => {
                self.visit_expression(&lhs)?;
                self.visit_expression(&rhs)?;

                match operator {
                    Operator::Add => self.context.emit_op(Op::Add),
                    Operator::Sub => self.context.emit_op(Op::Subtract),
                    Operator::Mul => self.context.emit_op(Op::Multiply),
                    Operator::Div => self.context.emit_op(Op::Divide),
                }
            }
            Expression::Integer(val) => self.context.emit_stack_push_int(*val),
            Expression::String(value) => self
                .context
                .emit_wrapped_op(WrappedOp::PushString { value }),
            Expression::Bool(x) => self.context.push_bool(*x),
            Expression::Variable(name) => {
                // FIXME: return err, instead of unwrap.
                self.context.emit_op(Op::GetLocal {
                    index: *self.variables.get(name).unwrap(),
                })
            }
        }

        Ok(())
    }

    fn visit_statement(&mut self, v: &'ast crate::ast::Statement) -> Result<(), Self::Error> {
        match v {
            Statement::Variable {
                name,
                var_type,
                value,
            } => {
                // Emit expression code.
                self.visit_expression(&value)?;

                // Emit a coerce operation.
                match var_type {
                    Type::Any => self.context.emit_op(Op::CoerceA),
                    _ => todo!(),
                }

                // Emit wrapped SetLocal op.
                self.context.emit_op(Op::SetLocal {
                    index: *self.variables.get(name).unwrap(),
                })
            }
        }

        Ok(())
    }

    fn visit_function(&mut self, v: &'ast crate::ast::Function) -> Result<(), Self::Error> {
        // This the setup that the asc.jar/Flex compiler always does in a function, so we do the same.
        self.context.emit_op(Op::GetLocal { index: 0 });
        self.context.emit_op(Op::PushScope);

        // Parse all statements.
        for statement in &v.block {
            self.visit_statement(&statement)?;
        }

        // Check if return type is void and use stuff accordingly.
        // FIXME: What if return keyword is used explicitly, then we need to clean it up somewhere. (Otherwise duplicates occur).
        if v.return_type == Type::Void {
            self.context.emit_op(Op::ReturnVoid);
        } else {
            // Not going to work yet.
            todo!()
        }

        Ok(())
    }
}
