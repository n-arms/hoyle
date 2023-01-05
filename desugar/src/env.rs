use ir::desugared::*;
use ir::qualified::Identifier;
use std::collections::HashMap;

pub struct Metadata {
    size: Atom,
}

pub struct Env<'ident> {
    metadata: HashMap<Identifier<'ident>, Metadata>,
    bindings: HashMap<Identifier<'ident>, Name>,
    functions: HashMap<Identifier<'ident>, ()>,
}

impl<'ident> Env<'ident> {
    pub fn new(
        metadata: HashMap<Identifier<'ident>, Metadata>,
        bindings: HashMap<Identifier<'ident>, Name>,
        functions: HashMap<Identifier<'ident>, ()>,
    ) -> Self {
        Self {
            metadata,
            bindings,
            functions,
        }
    }

    pub fn define_identifier(&mut self, identifier: Identifier<'ident>, name: Name) {
        assert!(!self.bindings.contains_key(&identifier));
        self.bindings.insert(identifier, name);
    }

    pub fn lookup_identifier(&self, identifier: &Identifier<'ident>) -> Name {
        *self
            .bindings
            .get(identifier)
            .expect("the frontend should have caught undefined variables")
    }
}
