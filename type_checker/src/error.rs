use ir::typed::Type;
use std::result;

#[derive(Debug)]
pub enum Error<'expr, 'ident> {
    TypeMismatch {
        expected: Type<'expr, 'ident>,
        found: Type<'expr, 'ident>,
    },
}

pub type Result<'expr, 'ident, T> = result::Result<T, Error<'expr, 'ident>>;
