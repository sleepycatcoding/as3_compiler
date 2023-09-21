//! The purpose of this module is to generate ABC bytecode for functions.

use swf::avm2::types::{Index, MethodBody, Op};

use crate::ast::{
    visitor::{walk_function, walk_statement},
    Expression, Operator, Statement, Type, Visitor,
};
use crate::codegen::CodeGenerationContext;
use std::collections::HashMap;

/// A visitor used to collect and assign indices to all variables/arguments referenced in a function.
#[derive(Debug)]
pub struct LocalResolverVisitor<'a> {
    current_index: u32,
    // FIXME: Maybe store type here too since it might be needed later?
    variables: HashMap<&'a String, u32>,
}

impl Default for LocalResolverVisitor<'_> {
    fn default() -> Self {
        Self {
            current_index: 0,
            variables: HashMap::new(),
        }
    }
}

impl<'a> LocalResolverVisitor<'a> {
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

impl<'ast> Visitor<'ast> for LocalResolverVisitor<'ast> {
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
    context: &'a mut CodeGenerationContext,
    locals: HashMap<&'ast String, u32>,
    // Max stack objects at once at any time during execution.
    max_stack: u32,
    current_stack: u32,
    /// Generated code.
    code: Vec<Op>,
}

impl<'ast, 'a> FunctionGenerator<'ast, 'a> {
    pub fn new(
        variables: LocalResolverVisitor<'ast>,
        context: &'a mut CodeGenerationContext,
    ) -> Self {
        Self {
            context,
            locals: variables.variables,
            code: Vec::new(),
            max_stack: 0,
            current_stack: 0,
        }
    }

    // Meant to be used to emit raw operations, if possible, use wrappers instead.
    fn emit_op(&mut self, op: Op) {
        self.code.push(op);
    }

    /// A marker function to calculate a function's max stack value.
    fn stack_push(&mut self) {
        self.current_stack += 1;

        if self.max_stack < self.current_stack {
            self.max_stack = self.current_stack
        }
    }

    /// A marker function to calculate a function's max stack value.
    fn stack_pop(&mut self) {
        self.current_stack -= 1;
    }

    fn push_int(&mut self, val: i32) {
        if let Ok(value) = val.try_into() {
            self.emit_op(Op::PushByte { value })
        } else if let Ok(value) = val.try_into() {
            self.emit_op(Op::PushShort { value })
        } else {
            todo!("Unhandled");
        }

        self.stack_push()
    }

    fn push_string(&mut self, val: &String) {
        let index = self.context.add_string(val);
        self.emit_op(Op::PushString { value: index });
        self.stack_push()
    }

    fn push_bool(&mut self, val: bool) {
        if val {
            self.emit_op(Op::PushTrue)
        } else {
            self.emit_op(Op::PushFalse)
        }
        self.stack_push()
    }

    // Gets a value in a local register and pushes it onto the stack.
    fn get_local(&mut self, name: &String) {
        self.emit_op(Op::GetLocal {
            // FIXME: return err, instead of unwrap.
            index: *self.locals.get(name).unwrap(),
        });
        self.stack_push()
    }

    /// Pops a value from the stack and sets it to the specified local register.
    fn set_local(&mut self, name: &String) {
        self.emit_op(Op::SetLocal {
            index: *self.locals.get(name).unwrap(),
        });
        self.stack_pop();
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
                    Operator::Add => self.emit_op(Op::Add),
                    Operator::Sub => self.emit_op(Op::Subtract),
                    Operator::Mul => self.emit_op(Op::Multiply),
                    Operator::Div => self.emit_op(Op::Divide),
                }

                // These operations pop two values off the stack and pushes one back. So we pop only once.
                self.stack_pop();
            }
            Expression::Integer(val) => self.push_int(*val),
            Expression::String(val) => self.push_string(val),
            Expression::Bool(x) => self.push_bool(*x),
            Expression::Variable(name) => self.get_local(name),
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

                self.set_local(name);
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

        let method_body = MethodBody {
            // FIXME: This actually not correct.
            method: Index::new(0),
            // FIXME: Not yet calculated.
            max_stack: self.max_stack,
            // NOTE: Plus one required since we are not including "this" object in locals.
            num_locals: self.locals.len() as u32 + 1,
            // FIXME: No idea about these two.
            init_scope_depth: 0,
            max_scope_depth: 0,
            // FIXME: Swf crate does not provide an easy way to write opcodes out right now.
            //        also this requires jump/label resolution.
            code: Vec::new(),
            exceptions: Vec::new(),
            traits: Vec::new(),
        };

        println!("Method Body: {:?}", method_body);

        Ok(())
    }
}
