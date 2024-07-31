use im::HashMap;
use ir::bridge::{Variable, Witness};
use ir::name_source::NameSource;
use tree::typed::Type;
use tree::String;

#[derive(Clone)]
pub struct Env {
    pub name_source: NameSource,
    witnesses: HashMap<String, Witness>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            name_source: NameSource::default(),
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

    pub fn fresh_variable(&mut self, typ: Type, witness: Witness) -> Variable {
        self.define_variable(self.fresh_name(), typ, witness)
    }

    pub fn fresh_name(&self) -> String {
        self.name_source.fresh_name()
    }

    pub fn lookup_witness(&self, name: &String) -> Witness {
        self.witnesses.get(name).cloned().unwrap()
    }
}
