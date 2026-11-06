use crate::ports::Tokens;
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Default)]
pub struct Sequence(pub Rc<Cell<u32>>);
impl Tokens for Sequence {
    fn next(&mut self) -> String {
        self.0.set(self.0.get() + 1);
        self.0.get().to_string()
    }
}
