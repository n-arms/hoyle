use crate::qualified::{Identifier, Tag, Type};
use std::collections::HashMap;

#[derive(Copy, Clone)]
pub struct Function<'expr, 'ident> {
    pub generic_type: Type<'expr, 'ident>,
    pub generic_args: &'expr [Identifier<'ident>],
}

#[derive(Copy, Clone)]
pub struct Struct {
    pub metadata_constructor: Tag,
}

#[derive(Default)]
pub struct Metadata<'expr, 'ident> {
    pub functions: HashMap<Identifier<'ident>, Function<'expr, 'ident>>,
    pub structs: HashMap<Identifier<'ident>, Struct>,
}

impl<'expr, 'ident> Metadata<'expr, 'ident> {
    pub fn merge(&mut self, other: &Self) {
        self.functions.extend(other.functions.iter());
        self.structs.extend(other.structs.iter());
    }
}
