use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
pub struct DynamicVec(pub Rc<RefCell<Vec<u8>>>);

impl std::io::Write for DynamicVec {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
