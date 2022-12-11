use bumpalo::Bump;
use im::HashMap;
use infinite_iterator::InfiniteIterator;
use ir::ast::*;
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct TypedId<'expr, ID> {
    pub id: ID,
    pub id_type: Type<'expr, ID>,
}

impl<'expr, ID> TypedId<'expr, ID> {
    pub fn new(id: ID, id_type: Type<'expr, ID>) -> Self {
        Self { id, id_type }
    }
}

#[derive(Clone, Default)]
pub struct Env<'new, ID> {
    variable_types: HashMap<ID, TypedId<'new, ID>>,
}

impl<'new, ID> Env<'new, ID>
where
    ID: Hash + Eq + Clone,
{
    pub fn bind_variable(&self, variable: ID, variable_type: Type<'new, ID>) -> Self {
        let mut result = self.clone();
        result
            .variable_types
            .insert(variable.clone(), TypedId::new(variable, variable_type));
        result
    }
}

pub struct Fresh<ID, S> {
    source: S,
    _marker: PhantomData<ID>,
}

impl<ID, S> Fresh<ID, S>
where
    S: InfiniteIterator<Item = ID>,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            _marker: PhantomData::default(),
        }
    }

    pub fn fresh(&mut self) -> ID {
        self.source.next_infinite()
    }
}

pub struct Alloc<'new> {
    bump: &'new Bump,
}

impl<'new> Alloc<'new> {
    pub fn new(bump: &'new Bump) -> Self {
        Self { bump }
    }

    pub fn alloc<T>(&self, t: T) -> &'new T {
        self.bump.alloc(t)
    }

    pub fn alloc_str<'a>(&self, string: &'a str) -> &'new str {
        self.bump.alloc_str(string)
    }

    pub fn alloc_slice_copy<'a, T: Copy>(&self, slice: &[T]) -> &'new [T] {
        self.bump.alloc_slice_copy(slice)
    }
}
