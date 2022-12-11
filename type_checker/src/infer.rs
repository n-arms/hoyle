use crate::env::*;
use bumpalo::Bump;
use infinite_iterator::InfiniteIterator;
use ir::ast::*;
use std::hash::Hash;

pub fn program<'old, 'new, ID, S>(
    to_infer: &Program<'old, ID>,
    env: Env<'new, ID>,
    fresh: &mut Fresh<ID, S>,
    alloc: &'new Bump,
) -> Program<'new, TypedId<'new, ID>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn definition<'old, 'new, ID, S>(
    to_infer: &Definition<'old, ID>,
    env: Env<'new, ID>,
    fresh: &mut Fresh<ID, S>,
    alloc: &'new Bump,
) -> Definition<'new, TypedId<'new, ID>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn statement<'old, 'new, ID, S>(
    to_infer: &Statement<'old, ID>,
    env: Env<'new, ID>,
    fresh: &mut Fresh<ID, S>,
    alloc: &'new Bump,
) -> Statement<'new, TypedId<'new, ID>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn expr<'old, 'new, ID, S>(
    to_infer: &Expr<'old, ID>,
    env: Env<'new, ID>,
    fresh: &mut Fresh<ID, S>,
    alloc: &'new Bump,
) -> Statement<'new, TypedId<'new, ID>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn pattern<'old, 'new, ID, S>(
    to_infer: &Pattern<'old, ID>,
    env: Env<'new, ID>,
    fresh: &mut Fresh<ID, S>,
    alloc: &'new Bump,
) -> Pattern<'new, TypedId<'new, ID>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn r#type<'old, 'new, ID, S>(
    to_infer: &Type<'old, ID>,
    env: Env<'new, ID>,
    fresh: &mut Fresh<ID, S>,
    alloc: &'new Bump,
) -> Pattern<'new, TypedId<'new, ID>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}
