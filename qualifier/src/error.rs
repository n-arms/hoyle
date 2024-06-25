use ir::qualified::{Field, FieldDefinition};
use ir::source::{self, PatternField};
use smartstring::{LazyCompact, SmartString};
use std::result;

#[derive(Clone, Debug)]
pub enum Error<'old, 'new> {
    UndefinedVariable(source::Identifier),
    UndefinedType(source::Identifier),
    UndefinedStruct(source::Identifier),
    StructLiteralMissingField(FieldDefinition<'new>, &'new [Field<'new>]),
    StructLiteralContainsExtraField(source::Field<'old>, &'new [FieldDefinition<'new>]),
    StructPatternMissingField(PatternField<'old>, &'new [FieldDefinition<'new>]),
}

pub type Result<'old, 'new, T> = result::Result<T, Error<'old, 'new>>;
