use crate::ast;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct Identifier<'expr, 'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
    pub r#type: Option<Type<'expr, 'ident, ast::Span>>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TypeName<'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
}

#[derive(Copy, Clone)]
pub struct StructDefinition<'expr, 'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
    pub fields: &'expr [FieldDefinition<'expr, 'ident>],
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Path {
    Current,
    Builtin,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum IdentifierSource {
    Local,
    Global(Path),
}

#[derive(Copy, Clone)]
pub enum Type<'expr, 'ident, SPAN> {
    Named {
        name: TypeName<'ident>,
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
    pub name: &'ident str,
    pub field_type: Type<'expr, 'ident, SPAN>,
    pub span: SPAN,
}

pub type Program<'expr, 'ident> =
    ast::Program<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type FieldDefinition<'expr, 'ident> =
    ast::FieldDefinition<'ident, Type<'expr, 'ident, ast::Span>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, 'ident, Identifier<'expr, 'ident>>;

pub type Block<'expr, 'ident> =
    ast::Block<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Field<'expr, 'ident> =
    ast::Field<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Branch<'expr, 'ident> =
    ast::Branch<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type Expr<'expr, 'ident> =
    ast::Expr<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident, ast::Span>>;

pub type PatternField<'expr, 'ident> = ast::PatternField<'expr, 'ident, Identifier<'expr, 'ident>>;

impl Debug for Identifier<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}::{} : {:?}", self.source, self.name, self.r#type)
    }
}

impl Debug for TypeName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}::{}", self.source, self.name)
    }
}

impl Debug for IdentifierSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Global(path) => path.fmt(f),
        }
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Current => write!(f, "current"),
            Self::Builtin => write!(f, "builtin"),
        }
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
