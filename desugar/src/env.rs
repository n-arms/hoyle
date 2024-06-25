use ir::desugared::Type;
use ir::metadata::{Function, Metadata};
use ir::qualified::{Identifier, Primitives, Tag};
use std::collections::HashMap;

pub struct Env<'old, 'new, 'meta> {
    metadata: &'meta Metadata<'old>,
    rebound_generics: HashMap<Tag, Type<'new>>,
    pub primitives: Primitives,
    pub metadata_type: Type<'new>,
}

impl<'old, 'new, 'meta> Env<'old, 'new, 'meta> {
    #[must_use]
    pub fn new(
        metadata: &'meta Metadata<'old>,
        primitives: Primitives,
        metadata_type: Type<'new>,
    ) -> Self {
        Self {
            metadata,
            primitives,
            metadata_type,
            rebound_generics: HashMap::new(),
        }
    }

    #[must_use]
    pub fn lookup_variable(&self, variable: Identifier) -> Option<Function<'old>> {
        self.metadata.functions.get(&variable).copied()
    }

    pub fn bind_generic(&mut self, generic: Tag, rebound_type: Type<'new>) {
        self.rebound_generics.insert(generic, rebound_type);
    }

    #[must_use]
    pub fn lookup_generic(&self, generic: Tag) -> Type<'new> {
        self.rebound_generics
            .get(&generic)
            .copied()
            .unwrap_or(Type::Named { name: generic })
    }
}
