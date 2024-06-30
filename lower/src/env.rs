use std::cell::Cell;
use std::rc::Rc;

use im::HashMap;
use ir::bridge::{Size, Variable};
use tree::typed::Type;
use tree::{sized, String};

#[derive(Clone)]
pub struct Env {
    pub next_name: Rc<Cell<usize>>,
    variables: HashMap<String, VariableScheme>,
}

#[derive(Clone)]
pub struct VariableScheme {
    pub variable: Variable,
    pub size: sized::Size,
    pub witness: Option<sized::Expr>,
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
        size: sized::Size,
        witness: Option<sized::Expr>,
    ) -> Variable {
        let variable = Variable {
            name: name.clone(),
            typ,
        };
        let scheme = VariableScheme {
            variable: variable.clone(),
            size,
            witness,
        };
        self.variables.insert(name, scheme);
        variable
    }

    pub fn lookup_variable_scheme(&self, name: &String) -> &VariableScheme {
        self.variables
            .get(name)
            .expect(&format!("Unknown variable {}", name))
    }

    pub fn lookup_variable(&self, name: &String) -> Variable {
        self.lookup_variable_scheme(name).variable.clone()
    }

    pub fn fresh_name(&mut self) -> String {
        let name = self.next_name.take();
        self.next_name.set(name + 1);
        String::from(format!("_{}", name))
    }

    pub fn lookup_witness(&self, name: &String) -> Option<sized::Expr> {
        self.lookup_variable_scheme(name).witness.clone()
    }
}
