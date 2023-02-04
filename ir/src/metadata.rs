use crate::qualified::{Identifier, Tag, Type};
use std::collections::HashMap;

pub struct Function<'expr, 'ident> {
    pub generic_type: Type<'expr, 'ident>,
    pub generic_args: &'expr [Identifier<'ident>],
}

pub struct Struct {
    pub metadata_constructor: Tag,
}

#[derive(Default)]
pub struct Metadata<'expr, 'ident> {
    pub functions: HashMap<Identifier<'ident>, Function<'expr, 'ident>>,
    pub structs: HashMap<Identifier<'ident>, Struct>,
}
