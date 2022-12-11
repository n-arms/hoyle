use bumpalo::Bump;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone)]
pub struct Identifier<'ident> {
    bump: &'ident Bump,
    interner: Rc<RefCell<HashSet<&'ident str>>>,
}

impl<'ident> Identifier<'ident> {
    fn get_or_intern<'a>(&self, string: &'a str) -> &'ident str {
        if let Some(interned) = self.interner.borrow().get(string) {
            interned
        } else {
            self.bump.alloc_str(string)
        }
    }

    fn new(bump: &'ident Bump) -> Self {
        Self {
            bump,
            interner: Rc::new(RefCell::new(HashSet::default())),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Ast<'expr> {
    pub bump: &'expr Bump,
}

#[derive(Clone)]
pub struct General<'ident, 'expr> {
    identifier: Identifier<'ident>,
    ast: Ast<'expr>,
}

impl<'ident, 'expr> General<'ident, 'expr> {
    pub fn new(identifier: &'ident Bump, ast: &'expr Bump) -> Self {
        Self {
            identifier: Identifier::new(identifier),
            ast: Ast { bump: ast },
        }
    }

    pub fn get_or_intern<'a>(&self, string: &'a str) -> &'ident str {
        self.identifier.get_or_intern(string)
    }

    pub fn ast_alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &'expr [T] {
        self.ast.bump.alloc_slice_copy(slice)
    }

    pub fn ast_alloc_str(&self, string: &str) -> &'expr str {
        self.ast.bump.alloc_str(string)
    }

    pub fn ast_alloc<T>(&self, ast: T) -> &'expr T {
        self.ast.bump.alloc(ast)
    }
}
