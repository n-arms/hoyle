use std::cell::RefCell;
use std::rc::Rc;

use im::HashMap;

use im::HashSet;
use tree::sized::*;
use tree::String;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructInstance {
    pub name: String,
}

#[derive(Clone, Default)]
pub struct Env {
    variables: HashMap<String, Variable>,
    structs: HashMap<String, Struct>,
    instances: Rc<RefCell<HashMap<StructInstance, ()>>>,
    trivial_types: HashSet<String>,
}

impl Env {
    pub fn define_variable(&mut self, variable: String, witness: Witness) {
        self.variables.insert(
            variable.clone(),
            Variable {
                name: variable,
                witness,
            },
        );
    }

    pub fn lookup_variable(&self, variable: &String) -> Variable {
        self.variables
            .get(variable)
            .expect(&format!("unknown variable {variable}"))
            .clone()
    }

    pub fn define_struct(&mut self, name: String, definition: Struct) {
        self.structs.insert(name, definition);
    }

    pub fn lookup_struct(&self, name: &String) -> Struct {
        self.structs.get(name).unwrap().clone()
    }

    pub fn witness_struct_instance(&self, instance: StructInstance, witness: ()) {
        self.instances.borrow_mut().insert(instance, witness);
    }
}
