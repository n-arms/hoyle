use bumpalo::Bump;
use ir::ast::*;

pub struct TypedId<'expr, ID> {
    id: ID,
    id_type: Type<'expr, ID>,
}

pub fn type_check<'old, 'new, ID>(
    program: &Program<'old, ID>,
    alloc: &'new Bump,
) -> Program<'new, TypedId<'new, ID>> {
    todo!()
}
