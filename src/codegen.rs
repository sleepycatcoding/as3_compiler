use crate::ast::Visitor;
use swf::avm2::types::{ConstantPool, Index, Op};

mod function;

#[derive(Debug)]
pub struct CodeGenerationContext {
    /// Generated code.
    code: Vec<Op>,
    const_pool: ConstantPool,
}

impl CodeGenerationContext {
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

    /// Pushes a string onto the stack, if the string does not exist in the constant pool, it will be added.
    fn push_string(&mut self, val: &String) {
        let index = if let Some(index) = self.const_pool.strings.iter().position(|x| x == val) {
            index
        } else {
            // Create a index.
            let index = self.const_pool.strings.len();
            // Push the value.
            self.const_pool.strings.push(val.clone());
            index
        } as u32;

        let index = Index::new(index);

        self.emit_op(Op::PushString { value: index });
    }

    fn push_bool(&mut self, val: bool) {
        if val {
            self.emit_op(Op::PushTrue)
        } else {
            self.emit_op(Op::PushFalse)
        }
    }
}

/// A visitor to generate ABC bytecode from a AST.
#[derive(Debug)]
pub struct CodeGenerator {
    context: CodeGenerationContext,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self {
            context: CodeGenerationContext {
                code: Vec::new(),
                const_pool: ConstantPool {
                    ints: Vec::new(),
                    uints: Vec::new(),
                    doubles: Vec::new(),
                    strings: Vec::new(),
                    namespaces: Vec::new(),
                    namespace_sets: Vec::new(),
                    multinames: Vec::new(),
                },
            },
        }
    }
}

impl<'ast> Visitor<'ast> for CodeGenerator {
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
