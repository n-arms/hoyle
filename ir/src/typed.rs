use crate::ast;
use crate::qualified::{self, IdentifierSource, Type};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct Identifier<'expr, 'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
    pub r#type: Type<'expr, 'ident>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct UntypedIdentifier<'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
}

pub type Program<'expr, 'ident> =
    ast::Program<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, 'ident, Identifier<'expr, 'ident>>;

pub type Block<'expr, 'ident> =
    ast::Block<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Field<'expr, 'ident> =
    ast::Field<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Branch<'expr, 'ident> =
    ast::Branch<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Expr<'expr, 'ident> =
    ast::Expr<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type PatternField<'expr, 'ident> = ast::PatternField<'expr, 'ident, Identifier<'expr, 'ident>>;

impl Debug for Identifier<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)?;
        write!(f, "::")?;
        self.name.fmt(f)?;
        write!(f, " : ")?;
        self.r#type.fmt(f)
    }
}

impl<'ident> From<qualified::Identifier<'_, 'ident>> for UntypedIdentifier<'ident> {
    fn from(id: qualified::Identifier<'_, 'ident>) -> Self {
        Self {
            source: id.source,
            name: id.name,
        }
    }
}
