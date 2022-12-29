use std::result;

#[derive(Debug)]
pub enum Error<'ident> {
    UndefinedVariable(&'ident str),
    UndefinedType(&'ident str),
    UndefinedStruct(&'ident str),
}

pub type Result<'ident, T> = result::Result<T, Error<'ident>>;
