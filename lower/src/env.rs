use std::cell::Cell;
use std::rc::Rc;

use im::HashMap;
use ir::bridge::{Variable, Witness};
use tree::typed::Type;
use tree::{sized, String};

#[derive(Clone)]
pub struct Env {
    pub next_name: Rc<Cell<usize>>,
    witnesses: HashMap<String, Witness>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            next_name: Rc::new(Cell::new(0)),
            witnesses: HashMap::new(),
        }
    }

    pub fn define_variable(&mut self, name: String, typ: Type, witness: Witness) -> Variable {
        let variable = Variable {
            name: name.clone(),
            typ,
        };
        self.witnesses.insert(name, witness);
        variable
    }

    pub fn fresh_name(&mut self) -> String {
        let name = self.next_name.take();
        self.next_name.set(name + 1);
        String::from(format!("_{}", name))
    }

    pub fn lookup_witness(&self, name: &String) -> Witness {
        self.witnesses.get(name).cloned().unwrap()
    }
}
