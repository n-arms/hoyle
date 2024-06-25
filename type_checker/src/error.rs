use ir::qualified::Type;
use std::result;

#[derive(Clone, Debug)]
pub enum Error<'expr> {
    TypeMismatch {
        expected: Type<'expr>,
        found: Type<'expr>,
    },
}

pub type Result<'expr, T> = result::Result<T, Error<'expr>>;
