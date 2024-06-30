use im::HashMap;

use tree::sized::*;
use tree::String;

#[derive(Clone, Default)]
pub struct Env {
    variables: HashMap<String, Variable>,
    structs: HashMap<String, Struct>,
}

impl Env {
    pub fn define_variable(&mut self, variable: String, size: Size, witness: Expr) {
        self.variables.insert(
            variable.clone(),
            Variable {
                name: variable,
                size,
                witness: Box::new(witness),
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
}
