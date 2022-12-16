use crate::definitions::Definitions;
use crate::error::Result;
use arena_alloc::{Interning, Specialized};
use ir::ast;
use ir::qualified::*;

pub fn program<'old, 'new, 'ident>(
    to_qualify: ast::Program<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Program<'new, 'ident>> {
    todo!()
}

pub fn definition<'old, 'new, 'ident>(
    to_qualify: ast::Definition<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Definition<'new, 'ident>> {
    todo!()
}

pub fn argument<'old, 'new, 'ident>(
    to_qualify: ast::Argument<'old, &'ident str, ast::Type<'old, 'ident>>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Argument<'new, 'ident>> {
    todo!()
}

pub fn statement<'old, 'new, 'ident>(
    to_qualify: ast::Statement<'old, &'ident str, ast::Type<'old, 'ident>>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Statement<'new, 'ident>> {
    todo!()
}

pub fn pattern<'old, 'new, 'ident>(
    to_qualify: ast::Pattern<'old, &'ident str>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Pattern<'new, 'ident>> {
    todo!()
}

pub fn block<'old, 'new, 'ident>(
    to_qualify: ast::Block<'old, &'ident str, ast::Type<'old, 'ident>>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Block<'new, 'ident>> {
    todo!()
}

pub fn r#type<'old, 'new, 'ident>(
    to_qualify: ast::Type<'old, 'ident>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Type<'new, 'ident>> {
    todo!()
}

pub fn expr<'old, 'new, 'ident>(
    to_qualify: ast::Expr<'old, &'ident str, ast::Type<'old, 'ident>>,
    definitions: Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
) -> Result<'ident, Expr<'new, 'ident>> {
    todo!()
}
