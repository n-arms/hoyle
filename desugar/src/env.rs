use ir::desugared::*;
use ir::qualified::{self, Generic, Identifier};
use std::collections::HashMap;

pub struct Metadata {
    size: Atom,
    name: Name,
}

#[derive(Copy, Clone, Debug)]
pub enum VariableTemplate<'old, 'ident> {
    Monomorphic {
        name: Name,
    },
    Polymorphic {
        label: Label,
        definition: qualified::Type<'old, 'ident>,
        generics: &'old [Generic<'ident>],
    },
}

pub struct Env<'old, 'ident> {
    metadata: HashMap<Identifier<'ident>, Metadata>,
    variables: HashMap<Identifier<'ident>, VariableTemplate<'old, 'ident>>,
}

impl<'old, 'ident> Env<'old, 'ident> {
    pub fn new(metadata: HashMap<Identifier<'ident>, Metadata>) -> Self {
        let variables = HashMap::default();
        Self {
            metadata,
            variables,
        }
    }

    pub fn lookup_variable(&self, variable: Identifier<'ident>) -> VariableTemplate<'old, 'ident> {
        *self
            .variables
            .get(&variable)
            .expect("frontend should have caught undefined variables")
    }
}
