use crate::qualified::{Identifier, Tag, Type};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Function<'expr> {
    pub generic_type: Type<'expr>,
    pub generic_args: &'expr [Identifier],
}

#[derive(Clone)]
pub struct Struct {
    pub metadata_constructor: Tag,
}

#[derive(Default)]
pub struct Metadata<'expr> {
    pub functions: HashMap<Identifier, Function<'expr>>,
    pub structs: HashMap<Identifier, Struct>,
}

impl<'expr> Metadata<'expr> {
    pub fn merge(&mut self, other: &Self) {
        self.functions.extend(other.functions.clone());
        self.structs.extend(other.structs.clone());
    }
}
