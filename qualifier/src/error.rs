use std::result;

pub enum Error<'ident> {
    UndefinedVariable(&'ident str),
    UndefinedType(&'ident str),
}

pub type Result<'ident, T> = result::Result<T, Error<'ident>>;
