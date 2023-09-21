use crate::ast::Visitor;
use swf::avm2::types::Op;

mod function;

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

#[derive(Debug)]
pub struct CodeGenerationContext<'a> {
    /// Generated code.
    code: Vec<WrappedOp<'a>>,
}

impl<'a> CodeGenerationContext<'a> {
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

/// A visitor to generate ABC bytecode from a AST.
#[derive(Debug)]
pub struct CodeGenerator<'a> {
    context: CodeGenerationContext<'a>,
}

impl Default for CodeGenerator<'_> {
    fn default() -> Self {
        Self {
            context: CodeGenerationContext { code: Vec::new() },
        }
    }
}

impl<'ast> Visitor<'ast> for CodeGenerator<'ast> {
    type Error = ();

    fn visit_function(&mut self, v: &'ast crate::ast::Function) -> Result<(), Self::Error> {
        let mut var_visitor = function::VariableCollectVisitor::default();
        var_visitor.visit_function(v)?;

        let mut function_gen = function::FunctionGenerator::new(var_visitor, &mut self.context);
        function_gen.visit_function(v)?;

        Ok(())
    }

    fn visit_class(&mut self, _v: &'ast crate::ast::Class) -> Result<(), Self::Error> {
        todo!()
    }

    fn visit_package(&mut self, _v: &'ast crate::ast::Package) -> Result<(), Self::Error> {
        todo!()
    }
}
