use crate::ast;
use crate::qualified;
use std::fmt::{Debug, Formatter};

pub type Type<'expr, 'ident> = qualified::Type<'expr, 'ident, Option<ast::Span>>;

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
    ast::Program<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, 'ident, Identifier<'expr, 'ident>>;

pub type FieldDefinition<'expr, 'ident> = ast::FieldDefinition<'ident, Type<'expr, 'ident>>;

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
        self.identifier.fmt(f)?;
        write!(f, " : ")?;
        self.r#type.fmt(f)
    }
}
