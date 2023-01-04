use ir::ast::{self, PatternField};
use ir::qualified::{Field, FieldDefinition};
use std::result;

#[derive(Debug)]
pub enum Error<'old, 'new, 'ident> {
    UndefinedVariable(&'ident str),
    UndefinedType(&'ident str),
    UndefinedStruct(&'ident str),
    StructLiteralMissingField(FieldDefinition<'new, 'ident>, &'new [Field<'new, 'ident>]),
    StructLiteralContainsExtraField(
        ast::Field<'old, &'ident str, &'ident str>,
        &'new [FieldDefinition<'new, 'ident>],
    ),
    StructPatternMissingField(
        PatternField<'old, &'ident str>,
        &'new [FieldDefinition<'new, 'ident>],
    ),
}

pub type Result<'old, 'new, 'ident, T> = result::Result<T, Error<'old, 'new, 'ident>>;
