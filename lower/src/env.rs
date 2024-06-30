use std::cell::Cell;
use std::rc::Rc;

use im::HashMap;
use ir::bridge::{Size, Variable};
use tree::typed::Type;
use tree::String;

#[derive(Clone)]
pub struct Env {
    pub next_name: Rc<Cell<usize>>,
    variables: HashMap<String, Variable>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            next_name: Rc::new(Cell::new(0)),
            variables: HashMap::new(),
        }
    }

    pub fn from_name(name: Rc<Cell<usize>>) -> Self {
        Self {
            next_name: name,
            variables: HashMap::new(),
        }
    }

    pub fn allocate_variable(
        &mut self,
        name: String,
        typ: Type,
        size: Size,
        witness: Option<Variable>,
    ) -> Variable {
        let var = Variable {
            name: name.clone(),
            typ,
            size,
            witness: witness.map(Box::new),
        };
        self.variables.insert(name, var.clone());
        var
    }

    pub fn define_variable(&mut self, name: String, variable: Variable) {
        self.variables.insert(name, variable);
    }

    pub fn lookup_variable(&self, name: &String) -> Variable {
        self.variables
            .get(name)
            .cloned()
            .expect(&format!("Unknown variable {}", name))
    }

    pub fn fresh_name(&mut self) -> String {
        let name = self.next_name.take();
        self.next_name.set(name + 1);
        String::from(format!("_{}", name))
    }
}
