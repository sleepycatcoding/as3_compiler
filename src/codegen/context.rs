use crate::codegen::LabelIndex;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use swf::avm2::types::{ConstantPool, Index};

/// A context that is used when generating code for a function.
#[derive(Debug)]
pub struct FunctionGenContext<'a> {
    const_pool: &'a mut ConstantPoolContext,
    label_counter: u32,
    labels: HashMap<String, LabelIndex>,
}

impl FunctionGenContext<'_> {
    pub fn get_next_label(&mut self) -> LabelIndex {
        let index = LabelIndex(self.label_counter);
        self.label_counter += 1;
        index
    }

    pub fn get_or_create_label(&mut self, name: &str) -> LabelIndex {
        // This would ideally use the Entry API, but we can't since get_next_label needs mutability.
        if !self.labels.contains_key(name) {
            let index = self.get_next_label();
            self.labels.insert(name.to_owned(), index);
        }

        *self.labels.get(name).expect("Label should exist")
    }
}

impl<'a> Deref for FunctionGenContext<'a> {
    type Target = ConstantPoolContext;

    fn deref(&self) -> &Self::Target {
        self.const_pool
    }
}

impl<'a> DerefMut for FunctionGenContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.const_pool
    }
}

/// A context that is used when a constant pool is required.
#[derive(Debug)]
pub struct ConstantPoolContext {
    const_pool: ConstantPool,
}

impl ConstantPoolContext {
    /// Adds a string into constant pool (unless it already exists) and a index into the pool will be returned.
    pub fn add_string(&mut self, val: &String) -> Index<String> {
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

impl Default for ConstantPoolContext {
    fn default() -> Self {
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
}
