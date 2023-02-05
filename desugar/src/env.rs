use ir::desugared::Type;
use ir::metadata::*;
use ir::qualified::{Identifier, Primitives, Tag};
use std::collections::HashMap;

pub struct Env<'old, 'new, 'ident, 'meta> {
    metadata: &'meta Metadata<'old, 'ident>,
    rebound_generics: HashMap<Tag, Type<'new>>,
    pub primitives: Primitives<'ident>,
    pub metadata_type: Type<'new>,
}

impl<'old, 'new, 'ident, 'meta> Env<'old, 'new, 'ident, 'meta> {
    pub fn new(
        metadata: &'meta Metadata<'old, 'ident>,
        primitives: Primitives<'ident>,
        metadata_type: Type<'new>,
    ) -> Self {
        Self {
            metadata,
            primitives,
            metadata_type,
            rebound_generics: HashMap::new(),
        }
    }

    pub fn lookup_variable(&self, variable: Identifier<'ident>) -> Option<Function<'old, 'ident>> {
        self.metadata.functions.get(&variable).copied()
    }

    pub fn bind_generic(&mut self, generic: Tag, rebound_type: Type<'new>) {
        self.rebound_generics.insert(generic, rebound_type);
    }

    pub fn lookup_generic(&self, generic: Tag) -> Type<'new> {
        self.rebound_generics
            .get(&generic)
            .copied()
            .unwrap_or(Type::Named { name: generic })
    }
}
