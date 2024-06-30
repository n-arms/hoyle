use im::HashMap;
use ir::bridge::{Size, Variable};
use tree::typed::Type;
use tree::String;

#[derive(Clone)]
pub struct Env {
    next_var: Size,
    next_name: usize,
    variables: HashMap<String, Variable>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            next_var: Size {
                static_size: 0,
                dynamic: Vec::new(),
            },
            next_name: 0,
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
        let offset = self.next_var.clone();
        self.next_var.static_size += size.static_size;
        self.next_var.dynamic.extend(size.dynamic.iter().cloned());
        let var = Variable {
            name: name.clone(),
            typ,
            size,
            offset,
            witness: witness.map(Box::new),
        };
        self.variables.insert(name, var.clone());
        var
    }

    pub fn lookup_variable(&self, name: &String) -> Variable {
        self.variables.get(name).cloned().unwrap()
    }

    pub fn fresh_name(&mut self) -> String {
        let name = self.next_name;
        self.next_name += 1;
        String::from(format!("_{}", name))
    }
}
