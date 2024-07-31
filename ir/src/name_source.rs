use std::{cell::Cell, rc::Rc};

use tree::String;

#[derive(Clone, Default)]
pub struct NameSource {
    next: Rc<Cell<usize>>,
}

impl NameSource {
    pub fn fresh_name(&self) -> String {
        String::from(format!("_{}", self.fresh_id()))
    }

    fn fresh_id(&self) -> usize {
        let id = self.next.take();
        self.next.set(id + 1);
        id
    }
}
