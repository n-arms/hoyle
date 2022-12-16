use bumpalo::Bump;

// use 0 size structs as type level keys for what the allocator can do :eyes:

#[derive(Copy, Clone)]
pub struct General<'a> {
    arena: &'a Bump,
}

impl<'a> General<'a> {
    pub const fn new(arena: &'a Bump) -> Self {
        Self { arena }
    }

    #[must_use]
    pub fn alloc<T>(&self, obj: T) -> &'a mut T {
        self.arena.alloc(obj)
    }

    #[must_use]
    pub fn alloc_str<'b>(&self, string: &'b str) -> &'a mut str {
        self.arena.alloc_str(string)
    }

    #[must_use]
    pub fn alloc_slice_fill_iter<I, T>(&self, iter: I) -> &'a [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.arena.alloc_slice_fill_iter(iter)
    }

    pub fn alloc_slice_try_fill_iter<I, T, E>(&self, into_iter: I) -> Result<&'a mut [T], E>
    where
        I: IntoIterator<Item = Result<T, E>>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = into_iter.into_iter();
        let mut result = Ok(());
        let mem = self
            .arena
            .alloc_slice_fill_with(iter.len(), |_| match iter.next() {
                Some(Ok(elem)) => elem,
                Some(Err(error)) => {
                    result = Err(error);
                    unsafe { std::mem::zeroed() }
                }
                None => unreachable!(),
            });
        result.map(|_| mem)
    }
}
