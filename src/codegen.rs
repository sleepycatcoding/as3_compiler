use crate::ast::Visitor;
use swf::avm2::types::Op;

pub mod context;

mod function;

/// A simple label index.
#[derive(Debug, Clone, Copy)]
pub struct LabelIndex(u32);

/// A wrapped Opcode.
///
/// This will be converted to normal opcodes later when the code is passed through a label resolver.
#[derive(Debug)]
pub enum CompilerOp {
    /// These opcodes are written as is, and will not be transformed.
    Raw(Op),
    Label(LabelIndex),
    IfFalse(LabelIndex),
    IfTrue(LabelIndex),
}

/// A visitor to generate ABC bytecode from a AST.
#[derive(Debug)]
pub struct CodeGenerator {
    context: context::ConstantPoolContext,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self {
            context: Default::default(),
        }
    }
}

impl<'ast> Visitor<'ast> for CodeGenerator {
    type Error = ();

    fn visit_function(&mut self, v: &'ast crate::ast::Function) -> Result<(), Self::Error> {
        let mut var_visitor = function::LocalResolverVisitor::default();
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
