use crate::ast::{Expression, Operator, Statement, Type, Visitor};
use swf::avm2::types::Op;

pub struct CodeGenerator {}

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
}

/// Our compilers "MIR" representation.
///
/// This representation is used to assign variables, create constant pools and lots of other stuff.
#[derive(Debug)]
pub struct CodeBlock<'a> {
    /// Sub codeblocks of this codeblock.
    ///'
    /// The code in children should be emitted before code in this codeblock.
    children: Vec<CodeBlock<'a>>,
    /// The code of the current codeblock.
    code: Vec<WrappedOp<'a>>,
}

impl<'a> CodeBlock<'a> {
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

    fn add_child(&mut self, block: CodeBlock<'a>) {
        self.children.push(block)
    }
}

impl Default for CodeBlock<'_> {
    fn default() -> Self {
        Self {
            code: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl<'ast> Visitor<'ast> for CodeGenerator {
    type Error = ();
    type Ok = CodeBlock<'ast>;

    fn visit_expression(
        &mut self,
        v: &'ast crate::ast::Expression,
    ) -> Result<Self::Ok, Self::Error> {
        let mut code = CodeBlock::default();

        match v {
            Expression::BinaryOperation { lhs, operator, rhs } => {
                let lhs = self.visit_expression(&lhs)?;
                let rhs = self.visit_expression(&rhs)?;

                code.add_child(lhs);
                code.add_child(rhs);

                match operator {
                    Operator::Add => code.emit_op(Op::Add),
                    Operator::Sub => code.emit_op(Op::Subtract),
                    Operator::Mul => code.emit_op(Op::Multiply),
                    Operator::Div => code.emit_op(Op::Divide),
                }
            }
            Expression::Integer(val) => code.emit_stack_push_int(*val),
            _ => todo!(),
        }

        Ok(code)
    }

    fn visit_statement(&mut self, v: &'ast crate::ast::Statement) -> Result<Self::Ok, Self::Error> {
        let mut code = CodeBlock::default();

        match v {
            Statement::Variable {
                name,
                var_type,
                value,
            } => {
                // Emit expression code.
                let expr = self.visit_expression(&value)?;
                code.add_child(expr);

                // Emit a coerce operation.
                match var_type {
                    Type::Any => code.emit_op(Op::CoerceA),
                    _ => todo!(),
                }

                // Emit wrapped SetLocal op.
                code.emit_wrapped_op(WrappedOp::SetLocal { name })
            }
        }

        Ok(code)
    }

    fn visit_class(&mut self, v: &'ast crate::ast::Class) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn visit_function(&mut self, v: &'ast crate::ast::Function) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn visit_package(&mut self, v: &'ast crate::ast::Package) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
