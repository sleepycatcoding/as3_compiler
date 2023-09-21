use crate::ast::{Expression, Operator, Statement, Type, Visitor};
use swf::avm2::types::Op;

#[derive(Debug)]
pub enum WrappedOp<'a> {
    RawOp(Op),
    /// Represents an unresolved local, which has no number assigned.
    /// This will be assigned later in a separate pass and converted into a raw op.
    GetLocal {
        name: &'a str,
    },
    /// Represents an unresolved local, which has no number assigned.
    /// This will be assigned later in a separate pass and converted into a raw op.
    SetLocal {
        name: &'a str,
    },
    /// Represents an unresolved stack value, which has no constant pool index assigned.
    PushString {
        value: &'a str,
    },
}

/// A visitor to generate ABC bytecode from a AST.
#[derive(Debug)]
pub struct CodeGenerator<'a> {
    /// Generated code.
    code: Vec<WrappedOp<'a>>,
}

impl<'a> CodeGenerator<'a> {
    fn emit_op(&mut self, op: Op) {
        self.code.push(WrappedOp::RawOp(op));
    }

    fn emit_wrapped_op(&mut self, op: WrappedOp<'a>) {
        self.code.push(op)
    }

    fn emit_stack_push_int(&mut self, val: i32) {
        if let Ok(value) = val.try_into() {
            self.emit_op(Op::PushByte { value })
        } else if let Ok(value) = val.try_into() {
            self.emit_op(Op::PushShort { value })
        } else {
            todo!("Unhandled");
        }
    }
}

impl Default for CodeGenerator<'_> {
    fn default() -> Self {
        Self { code: Vec::new() }
    }
}

impl<'ast> Visitor<'ast> for CodeGenerator<'ast> {
    type Error = ();

    fn visit_expression(&mut self, v: &'ast crate::ast::Expression) -> Result<(), Self::Error> {
        match v {
            Expression::BinaryOperation { lhs, operator, rhs } => {
                self.visit_expression(&lhs)?;
                self.visit_expression(&rhs)?;

                match operator {
                    Operator::Add => self.emit_op(Op::Add),
                    Operator::Sub => self.emit_op(Op::Subtract),
                    Operator::Mul => self.emit_op(Op::Multiply),
                    Operator::Div => self.emit_op(Op::Divide),
                }
            }
            Expression::Integer(val) => self.emit_stack_push_int(*val),
            Expression::String(value) => self.emit_wrapped_op(WrappedOp::PushString { value }),
            Expression::Variable(name) => self.emit_wrapped_op(WrappedOp::GetLocal { name }),
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
                    Type::Any => self.emit_op(Op::CoerceA),
                    _ => todo!(),
                }

                // Emit wrapped SetLocal op.
                self.emit_wrapped_op(WrappedOp::SetLocal { name })
            }
        }

        Ok(())
    }

    fn visit_function(&mut self, v: &'ast crate::ast::Function) -> Result<(), Self::Error> {
        // This the setup that the asc.jar/Flex compiler always does in a function, so we do the same.
        self.emit_op(Op::GetLocal { index: 0 });
        self.emit_op(Op::PushScope);

        // Parse all statements.
        for statement in &v.block {
            self.visit_statement(&statement)?;
        }

        // Check if return type is void and use stuff accordingly.
        // FIXME: What if return keyword is used explicitly, then we need to clean it up somewhere. (Otherwise duplicates occur).
        if v.return_type == Type::Void {
            self.emit_op(Op::ReturnVoid);
        } else {
            // Not going to work yet.
            todo!()
        }

        Ok(())
    }

    fn visit_class(&mut self, v: &'ast crate::ast::Class) -> Result<(), Self::Error> {
        todo!()
    }

    fn visit_package(&mut self, v: &'ast crate::ast::Package) -> Result<(), Self::Error> {
        todo!()
    }
}
