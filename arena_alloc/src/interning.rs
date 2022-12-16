use crate::general::General;
use bumpalo::Bump;
use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct GeneralPurpose;
pub struct Specialized;

#[derive(Clone)]
pub struct Interning<'a, USE> {
    general: General<'a>,
    interned: Rc<RefCell<HashSet<&'a str>>>,

    _marker: PhantomData<USE>,
}

impl<'a, USE> Interning<'a, USE> {
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            general: General::new(arena),
            interned: Rc::default(),
            _marker: PhantomData::default(),
        }
    }

    #[must_use]
    pub fn get_or_intern<'b>(&self, string: &'b str) -> &'a str {
        let mut interned = self.interned.borrow_mut();
        if let Some(result) = interned.get(string) {
            *result
        } else {
            let result = self.general.alloc_str(string);
            interned.insert(result);
            result
        }
    }
}

impl<'a> AsRef<General<'a>> for Interning<'a, GeneralPurpose> {
    fn as_ref(&self) -> &General<'a> {
        &self.general
    }
}
