use crate::codegen::assembler::intern::{Immediate, Intern, InternContext};
use std::collections::HashMap;
use swf::avm2::{
    types::{Index, Multiname, Op as SwfOp},
    write::Writer,
};

pub mod intern;
mod utils;

#[derive(Debug)]
enum Op {
    Raw(SwfOp),
    /// Wrapped label containing the labels index.
    Label(u32),
    IfTrue(u32),
    IfFalse(u32),
    IfEq(u32),
    Jump(u32),
}

impl From<SwfOp> for Op {
    fn from(value: SwfOp) -> Self {
        Op::Raw(value)
    }
}

#[derive(Debug)]
pub struct Label(u32);

impl Label {
    /// Links the current label to after the current opcode and emits an `Label` opcode.
    pub fn link(&self, assembler: &mut Assembler) {
        assembler.emit(Op::Label(self.0))
    }
}

#[derive(Debug)]
pub struct Assembler {
    opcodes: Vec<Op>,
    next_label_index: u32,
}

// Internal API.
impl Assembler {
    #[inline]
    fn emit(&mut self, op: impl Into<Op>) {
        self.opcodes.push(op.into());
    }

    // Used by opcode emitter functions to mark a change in stack value count. these are no-op for now.
    fn marker_push(&mut self) {}
    fn marker_pop(&mut self) {}

    /// Resolves all created labels to offsets in bytecode.
    fn resolve_labels(&self) -> HashMap<u32, u32> {
        let mut label_offsets = HashMap::new();

        // FIXME: Very ugly hack to access vec for length purposes.
        let buf = utils::DynamicVec::default();
        let buf_ref = buf.0.clone();
        let mut writer = Writer::new(buf);

        for op in &self.opcodes {
            match op {
                Op::Raw(x) => writer.write_op(x),
                Op::IfTrue(_) => writer.write_op(&SwfOp::IfTrue { offset: 0 }),
                Op::IfFalse(_) => writer.write_op(&SwfOp::IfFalse { offset: 0 }),
                Op::IfEq(_) => writer.write_op(&SwfOp::IfEq { offset: 0 }),
                Op::Jump(_) => writer.write_op(&SwfOp::Jump { offset: 0 }),
                Op::Label(x) => {
                    let cur_pos = buf_ref.borrow().len();
                    if let Some(_) = label_offsets.insert(*x, cur_pos as u32) {
                        // FIXME: Make this an error instead.
                        panic!("Same label was linked twice.");
                    }
                    writer.write_op(&SwfOp::Label)
                }
            }
            .expect("Should not fail");
        }

        label_offsets
    }
}

// Public API surface.
impl Assembler {
    pub fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            next_label_index: 0,
        }
    }

    /// Creates a new label, this has to be linked using [`Label::link()`]
    #[must_use]
    pub fn make_label(&mut self) -> Label {
        let label = Label(self.next_label_index);
        self.next_label_index += 1;
        label
    }

    /// Pops top most value on stack and checks if it is true and jumps if it is.
    pub fn jump_if_true(&mut self, label: &Label) {
        self.marker_pop();
        self.emit(Op::IfTrue(label.0))
    }

    pub fn jump_if_false(&mut self, label: &Label) {
        self.marker_pop();
        self.emit(Op::IfFalse(label.0))
    }

    pub fn jump_if_eq(&mut self, label: &Label) {
        self.marker_pop();
        self.marker_pop();
        self.emit(Op::IfEq(label.0))
    }

    /// Emits a jump to specified label.
    pub fn jump(&mut self, label: &Label) {
        self.emit(Op::Jump(label.0))
    }

    /// Interns and emits a push for specified value.
    pub fn push_interned<T: Intern>(&mut self, context: &mut InternContext, value: T) {
        self.marker_push();
        value.push(context, self)
    }

    /// Emits a push for specified value.
    pub fn push_immediate<T: Immediate>(&mut self, value: T) {
        self.marker_push();
        value.push(self)
    }

    // FIXME: Maybe unify such push operations?
    pub fn push_scope(&mut self) {
        self.marker_push();
        self.emit(SwfOp::PushScope)
    }
    pub fn get_global_scope(&mut self) {
        self.marker_push();
        self.emit(SwfOp::GetGlobalScope);
    }

    /// Swaps two top most stack values.
    pub fn swap(&mut self) {
        self.emit(SwfOp::Swap)
    }

    /// Gets the value from specified local register and pushes it onto the stack.
    pub fn get_local(&mut self, index: u32) {
        self.marker_push();
        self.emit(SwfOp::GetLocal { index })
    }
    /// Pops a value from the stack and sets it to the specified local register.
    pub fn set_local(&mut self, index: u32) {
        self.marker_pop();
        self.emit(SwfOp::SetLocal { index })
    }

    /// Pops a value from stack and gets the specified slot, which then is pushed back.
    pub fn get_slot(&mut self, index: u32) {
        // NOTE: No marker necessary, op leaves stack state same.
        self.emit(SwfOp::GetSlot { index })
    }
    /// Pops two values from stack where the first one is the object and second is the value.
    pub fn set_slot(&mut self, index: u32) {
        self.marker_pop();
        self.marker_pop();
        self.emit(SwfOp::SetSlot { index })
    }

    /// Emits a return void opcode.
    pub fn return_void(&mut self) {
        self.emit(SwfOp::ReturnVoid)
    }

    /// Pops a object, optional namespace/name and num_args of arguments, then calls the function.
    pub fn call_property(&mut self, name: Index<Multiname>, num_args: u32) {
        self.marker_pop();
        // FIXME: Take optional namespace/name into account.
        for _ in 0..num_args {
            self.marker_pop()
        }
        self.emit(SwfOp::CallProperty {
            index: name,
            num_args,
        })
    }

    /// Pops a function and num_args of arguments and then calls the function.
    pub fn call(&mut self, num_args: u32) {
        self.marker_pop();
        for _ in 0..num_args {
            self.marker_pop()
        }
        self.emit(SwfOp::Call { num_args })
    }

    pub fn find_property_strict(&mut self, name: Index<Multiname>) {
        // FIXME: Take optional namespace/name into account. (should be a common func).
        self.marker_push();
        self.emit(SwfOp::FindPropStrict { index: name })
    }

    /// Generates raw bytecode for the current assembly, this operation consumes the assembler.
    pub fn compile(self) -> Vec<u8> {
        let label_offsets = self.resolve_labels();

        let mut out = vec![];
        let mut writer = Writer::new(&mut out);

        for op in &self.opcodes {
            match op {
                Op::Jump(x) => writer.write_op(&SwfOp::Jump {
                    offset: *label_offsets.get(x).unwrap() as i32,
                }),
                Op::IfTrue(x) => writer.write_op(&SwfOp::IfTrue {
                    offset: *label_offsets.get(x).unwrap() as i32,
                }),
                Op::IfFalse(x) => writer.write_op(&SwfOp::IfFalse {
                    offset: *label_offsets.get(x).unwrap() as i32,
                }),
                Op::IfEq(x) => writer.write_op(&SwfOp::IfEq {
                    offset: *label_offsets.get(x).unwrap() as i32,
                }),
                Op::Label(_) => writer.write_op(&SwfOp::Label),
                Op::Raw(x) => writer.write_op(x),
            }
            .expect("Should not fail");
        }

        out
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Assembler;

    #[test]
    fn simple_label_test() {
        let mut assembler = Assembler::default();

        let label = assembler.make_label();

        // Link label to current location.
        label.link(&mut assembler);
        // Jump to label.
        assembler.jump(&label);

        println!("{:?}", assembler);
        println!("{:?}", assembler.compile());
    }
}
