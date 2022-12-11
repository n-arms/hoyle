use bumpalo::Bump;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Identifier<'ident> {
    bump: &'ident Bump,
    interner: HashSet<&'ident str>,
}

impl<'ident> Identifier<'ident> {
    #[must_use]
    fn get_or_intern<'a>(&mut self, string: &'a str) -> &'ident str {
        if let Some(interned) = self.interner.get(string) {
            interned
        } else {
            self.bump.alloc_str(string)
        }
    }

    #[must_use]
    fn new(bump: &'ident Bump) -> Self {
        Self {
            bump,
            interner: HashSet::default(),
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
    #[must_use]
    pub fn new(identifier: &'ident Bump, ast: &'expr Bump) -> Self {
        Self {
            identifier: Identifier::new(identifier),
            ast: Ast { bump: ast },
        }
    }

    #[must_use]
    pub fn get_or_intern<'a>(&mut self, string: &'a str) -> &'ident str {
        self.identifier.get_or_intern(string)
    }

    #[must_use]
    pub fn ast_alloc_slice_copy<T: Copy>(&self, slice: &[T]) -> &'expr [T] {
        self.ast.bump.alloc_slice_copy(slice)
    }

    #[must_use]
    pub fn ast_alloc_str(&self, string: &str) -> &'expr str {
        self.ast.bump.alloc_str(string)
    }

    #[must_use]
    pub fn ast_alloc<T>(&self, ast: T) -> &'expr T {
        self.ast.bump.alloc(ast)
    }
}
