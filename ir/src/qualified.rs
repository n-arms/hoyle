use crate::ast;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag {
    pub module: u32,
    pub identifier: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'ident> {
    pub tag: Tag,
    pub name: &'ident str,
}

#[derive(Copy, Clone, Debug)]
pub struct StructDefinition<'expr, 'ident> {
    pub name: Identifier<'ident>,
    pub fields: &'expr [FieldDefinition<'expr, 'ident>],
}

impl<'expr, 'ident> StructDefinition<'expr, 'ident> {
    pub fn find_field(&self, name: &'ident str) -> Option<FieldDefinition<'expr, 'ident>> {
        self.fields.iter().find(|field| field.name == name).copied()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Path {
    Current,
    Builtin,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IdentifierSource {
    Local,
    Global(Path),
}

#[derive(Copy, Clone)]
pub enum Type<'expr, 'ident, SPAN> {
    Named {
        name: Identifier<'ident>,
        span: SPAN,
    },
    Arrow {
        arguments: &'expr [Type<'expr, 'ident, SPAN>],
        return_type: &'expr Type<'expr, 'ident, SPAN>,
        span: SPAN,
    },
}

#[derive(Copy, Clone)]
pub struct TypeField<'expr, 'ident, SPAN> {
    pub id: Identifier<'ident>,
    pub field_type: Type<'expr, 'ident, SPAN>,
    pub span: SPAN,
}

pub type Program<'expr, 'ident> =
    ast::Program<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type FieldDefinition<'expr, 'ident> =
    ast::FieldDefinition<'ident, Type<'expr, 'ident, ast::Span>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, 'ident, Identifier<'ident>>;

pub type Block<'expr, 'ident> =
    ast::Block<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Field<'expr, 'ident> =
    ast::Field<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Branch<'expr, 'ident> =
    ast::Branch<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Expr<'expr, 'ident> =
    ast::Expr<'expr, 'ident, Identifier<'ident>, Type<'expr, 'ident, ast::Span>>;

pub type PatternField<'expr, 'ident> = ast::PatternField<'expr, 'ident, Identifier<'ident>>;

impl Debug for Identifier<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}::{}", self.tag, self.name)
    }
}

impl<SPAN: Copy> Debug for Type<'_, '_, SPAN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named { name, .. } => name.fmt(f),
            Type::Arrow {
                arguments,
                return_type,
                ..
            } => f
                .debug_tuple("func")
                .field(arguments)
                .field(return_type)
                .finish(),
        }
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}.{}", self.module, self.identifier)
    }
}
