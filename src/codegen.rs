use crate::ast::Visitor;
use swf::avm2::types::{ConstantPool, Index, Op};

mod function;

#[derive(Debug)]
pub struct CodeGenerationContext {
    const_pool: ConstantPool,
}

impl CodeGenerationContext {
    /// Adds a string into constant pool (unless it already exists) and a index will be returned.
    fn add_string(&mut self, val: &String) -> Index<String> {
        let index = if let Some(index) = self.const_pool.strings.iter().position(|x| x == val) {
            index
        } else {
            // Create a index.
            let index = self.const_pool.strings.len();
            // Push the value.
            self.const_pool.strings.push(val.clone());
            index
        } as u32;

        Index::new(index)
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
