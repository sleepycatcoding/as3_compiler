use crate::codegen::assembler::Assembler;
use swf::avm2::types::{ConstantPool, Index, Multiname, Namespace, Op as SwfOp};

/// Generic trait for types that can be interned into a constant pool.
pub trait Intern {
    /// This function is expected to intern the value and emit the correct push opcode.
    fn push(&self, intern_context: &mut InternContext, assembler: &mut Assembler);
}

/// Generic trait for types that can be pushed onto the stack.
pub trait Immediate {
    /// This function is expected emit the correct push opcode.
    fn push(&self, assembler: &mut Assembler);
}

/// A context that manages a constant pool.
///
/// Values can be interned by their specific methods.
#[derive(Debug)]
pub struct InternContext {
    const_pool: ConstantPool,
}

impl InternContext {
    pub fn new() -> Self {
        Self {
            const_pool: ConstantPool {
                ints: Vec::new(),
                uints: Vec::new(),
                doubles: Vec::new(),
                strings: Vec::new(),
                namespaces: Vec::new(),
                namespace_sets: Vec::new(),
                multinames: Vec::new(),
            },
        }
    }

    /// Interns the a string into this constant pool.
    pub fn intern_string(&mut self, val: &str) -> Index<String> {
        let index = if let Some(index) = self.const_pool.strings.iter().position(|x| x == val) {
            index
        } else {
            // Create a index.
            let index = self.const_pool.strings.len();
            // Push the value.
            self.const_pool.strings.push(val.to_owned());
            index
        } as u32;

        Index::new(index)
    }

    pub fn intern_namespace(&mut self, val: &Namespace) -> Index<Namespace> {
        let index = if let Some(index) = self.const_pool.namespaces.iter().position(|x| x == val) {
            index
        } else {
            // Create a index.
            let index = self.const_pool.namespaces.len();
            // Push the value.
            self.const_pool.namespaces.push(val.to_owned());
            index
        } as u32;

        Index::new(index)
    }

    pub fn intern_multiname(&mut self, val: &Multiname) -> Index<Multiname> {
        let index = if let Some(index) = self.const_pool.multinames.iter().position(|x| x == val) {
            index
        } else {
            // Create a index.
            let index = self.const_pool.multinames.len();
            // Push the value.
            self.const_pool.multinames.push(val.to_owned());
            index
        } as u32;

        Index::new(index)
    }

    pub fn empty_namespaced_qname(&mut self, name: &str) -> Index<Multiname> {
        let empty = self.intern_string("");
        let namespace = self.intern_namespace(&Namespace::Package(empty));
        let name = self.intern_string(name);

        self.intern_multiname(&Multiname::QName { namespace, name })
    }
}

impl Default for InternContext {
    fn default() -> Self {
        Self::new()
    }
}

// Intern implementations.
impl Intern for &str {
    fn push(&self, intern_context: &mut InternContext, assembler: &mut Assembler) {
        let index = intern_context.intern_string(self);
        assembler.emit(SwfOp::PushString { value: index })
    }
}
impl Intern for String {
    fn push(&self, intern_context: &mut InternContext, assembler: &mut Assembler) {
        let index = intern_context.intern_string(self);
        assembler.emit(SwfOp::PushString { value: index })
    }
}

// Immediate implementions.
impl Immediate for u8 {
    fn push(&self, assembler: &mut Assembler) {
        assembler.emit(SwfOp::PushByte { value: *self })
    }
}
impl Immediate for bool {
    fn push(&self, assembler: &mut Assembler) {
        assembler.emit(if *self {
            SwfOp::PushTrue
        } else {
            SwfOp::PushFalse
        })
    }
}
