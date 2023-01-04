use crate::ast;
use crate::qualified::{self, Type};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct Identifier<'expr, 'ident> {
    pub identifier: qualified::Identifier<'ident>,
    pub r#type: Type<'expr, 'ident>,
}

impl<'expr, 'ident> Identifier<'expr, 'ident> {
    #[must_use]
    pub const fn new(
        identifier: qualified::Identifier<'ident>,
        r#type: Type<'expr, 'ident>,
    ) -> Self {
        Self { identifier, r#type }
    }
}

pub type Program<'expr, 'ident> =
    ast::Program<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, Identifier<'expr, 'ident>>;

pub type FieldDefinition<'expr, 'ident> =
    ast::FieldDefinition<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Block<'expr, 'ident> =
    ast::Block<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Field<'expr, 'ident> =
    ast::Field<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Branch<'expr, 'ident> =
    ast::Branch<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type Expr<'expr, 'ident> =
    ast::Expr<'expr, Identifier<'expr, 'ident>, qualified::Identifier<'ident>>;

pub type PatternField<'expr, 'ident> = ast::PatternField<'expr, Identifier<'expr, 'ident>>;

impl Debug for Identifier<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.identifier.fmt(f)?;
        write!(f, " : ")?;
        self.r#type.fmt(f)
    }
}
