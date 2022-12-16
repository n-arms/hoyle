use im::{HashMap, HashSet};
use infinite_iterator::InfiniteIterator;
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct TypedId<'expr, ID> {
    pub id: ID,
    pub id_type: Type<'expr, ID>,
}

#[derive(Clone, Debug)]
pub enum Type<'expr, ID> {
    Named(ID),
    Tuple(&'expr [Type<'expr, ID>]),
}

impl<'expr, ID: Hash> Hash for TypedId<'expr, ID> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<'expr, ID: Eq> PartialEq for TypedId<'expr, ID> {
    fn eq(&self, other: &Self) -> bool {
        if self.id != other.id {
            return false;
        }
        match (&self.id_type, &other.id_type) {
            (Type::Named(name1), Type::Named(name2)) => name1 == name2,
            (Type::Tuple(..), Type::Tuple(..)) => todo!(),
            _ => false,
        }
    }
}

impl<'expr, ID: Eq> Eq for TypedId<'expr, ID> {}

impl<'expr, ID> TypedId<'expr, ID> {
    pub fn new(id: ID, id_type: Type<'expr, ID>) -> Self {
        Self { id, id_type }
    }
}

#[derive(Clone, Default)]
pub struct Env<'new, ID> {
    variable_types: HashMap<ID, TypedId<'new, ID>>,
    generics: HashSet<ID>,
}

impl<'new, ID> Env<'new, ID>
where
    ID: Hash + Eq + Clone,
{
    pub fn bind_variable(mut self, variable: ID, variable_type: Type<'new, ID>) -> Self {
        self.variable_types
            .insert(variable.clone(), TypedId::new(variable, variable_type));
        self
    }

    pub fn bind_variables(mut self, bindings: impl IntoIterator<Item = TypedId<'new, ID>>) -> Self {
        self.variable_types
            .extend(bindings.into_iter().map(|id| (id.id.clone(), id)));
        self
    }

    pub fn declare_generics(mut self, generics: impl IntoIterator<Item = ID>) -> Self {
        self.generics.extend(generics);
        self
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
