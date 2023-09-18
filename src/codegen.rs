use crate::ast::{Expression, Operator, Visitor};
use swf::avm2::types::{AbcFile, ConstantPool, Op};

pub struct CodeGenerator {}

#[derive(Debug)]
pub struct CodeBlock {
    code: Vec<Op>,
}

impl CodeBlock {
    fn emit_op(&mut self, op: Op) {
        self.code.push(op);
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

    fn merge(&mut self, block: CodeBlock) {
        self.code.extend(block.code);
    }
}

impl Default for CodeBlock {
    fn default() -> Self {
        Self { code: Vec::new() }
    }
}

impl Visitor for CodeGenerator {
    type Error = ();
    type Ok = CodeBlock;

    fn visit_expression(&mut self, v: &crate::ast::Expression) -> Result<Self::Ok, Self::Error> {
        let mut code = CodeBlock::default();

        match v {
            Expression::BinaryOperation { lhs, operator, rhs } => {
                let lhs = self.visit_expression(&lhs)?;
                let rhs = self.visit_expression(&rhs)?;

                code.merge(lhs);
                code.merge(rhs);

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

    fn visit_class(&mut self, v: &crate::ast::Class) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn visit_function(&mut self, v: &crate::ast::Function) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn visit_package(&mut self, v: &crate::ast::Package) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn visit_statement(&mut self, v: &crate::ast::Statement) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
