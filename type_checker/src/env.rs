use bumpalo::Bump;
use im::{HashMap, HashSet};
use infinite_iterator::InfiniteIterator;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::zeroed;

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

#[derive(Copy, Clone)]
pub struct Alloc<'new> {
    bump: &'new Bump,
}

impl<'new> Alloc<'new> {
    pub fn new(bump: &'new Bump) -> Self {
        Self { bump }
    }

    pub fn object<T>(&self, t: T) -> &'new T {
        self.bump.alloc(t)
    }

    pub fn str<'a>(&self, string: &'a str) -> &'new str {
        self.bump.alloc_str(string)
    }

    pub fn slice_copy<'a, T: Copy>(&self, slice: &[T]) -> &'new [T] {
        self.bump.alloc_slice_copy(slice)
    }

    pub fn slice_fill_iter<T, I>(&self, iter: I) -> &'new [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.bump.alloc_slice_fill_iter(iter)
    }

    pub fn try_slice_fill_iter<T, I, E>(&self, iter: I) -> Result<&'new [T], E>
    where
        I: IntoIterator<Item = Result<T, E>>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut into_iter = iter.into_iter();
        let mut success = Ok(());
        // This allocates a block of memory that might potentially have zeroed values.
        // It is safe because for any block of memory to be zeroed,
        // `success` must become an error value, after which `mem` would not exit the function's scope.
        let mem: &_ =
            self.bump
                .alloc_slice_fill_with(into_iter.len(), |_| match into_iter.next() {
                    Some(Ok(elem)) => elem,
                    Some(Err(err)) => {
                        success = Err(err);
                        unsafe { zeroed() }
                    }
                    None => unreachable!(),
                });

        success.map(|_| mem)
    }
}
