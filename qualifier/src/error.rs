use ir::qualified::{Field, FieldDefinition};
use std::result;

#[derive(Debug)]
pub enum Error<'expr, 'ident> {
    UndefinedVariable(&'ident str),
    UndefinedType(&'ident str),
    UndefinedStruct(&'ident str),
    StructLiteralMissingField(
        FieldDefinition<'expr, 'ident>,
        &'expr [Field<'expr, 'ident>],
    ),
}

pub type Result<'expr, 'ident, T> = result::Result<T, Error<'expr, 'ident>>;
