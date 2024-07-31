use ir::bridge::Variable;
use ir::name_source::NameSource;
use tree::typed::Type;
use tree::String;

#[derive(Clone)]
pub struct Env {
    pub name_source: NameSource,
}

impl Env {
    pub fn new() -> Self {
        Self {
            name_source: NameSource::default(),
        }
    }

    pub fn define_variable(&mut self, name: String, typ: Type) -> Variable {
        let variable = Variable {
            name: name.clone(),
            typ,
        };
        variable
    }

    pub fn fresh_variable(&mut self, typ: Type) -> Variable {
        self.define_variable(self.fresh_name(), typ)
    }

    pub fn fresh_name(&self) -> String {
        self.name_source.fresh_name()
    }
}
