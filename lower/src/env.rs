use std::cell::Cell;
use std::rc::Rc;

use im::HashMap;
use ir::bridge::{Convention, Variable, Witness};
use tree::typed::Type;
use tree::String;

#[derive(Clone, Default)]
pub struct GlobalEnv {
    function_signatures: HashMap<String, Vec<Convention>>,
}
impl GlobalEnv {
    pub fn define_function(&mut self, name: String, convention: Vec<Convention>) {
        self.function_signatures.insert(name, convention);
    }
}

#[derive(Clone)]
pub struct Env {
    pub next_name: Rc<Cell<usize>>,
    witnesses: HashMap<String, Witness>,
    type_cache: HashMap<Type, Witness>,
    global: GlobalEnv,
}

impl Env {
    pub fn new(global: GlobalEnv) -> Self {
        Self {
            next_name: Rc::new(Cell::new(0)),
            witnesses: HashMap::new(),
            type_cache: HashMap::new(),
            global,
        }
    }

    pub fn try_define_variable(&mut self, name: String, typ: Type) -> Option<Variable> {
        let witness = self.type_cache.get(&typ)?;
        Some(self.define_variable(name, typ, witness.clone()))
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

    pub fn lookup_convention(&self, function: &String) -> &[Convention] {
        self.global
            .function_signatures
            .get(function)
            .unwrap()
            .as_slice()
    }
}
