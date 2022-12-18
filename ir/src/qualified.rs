use crate::ast;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct Identifier<'expr, 'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
    pub r#type: Type<'expr, 'ident>,
}

#[derive(Copy, Clone)]
pub struct TypeName<'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
}

#[derive(Copy, Clone)]
pub enum Path {
    Current,
    Builtin,
}

#[derive(Copy, Clone)]
pub enum IdentifierSource {
    Local,
    Global(Path),
}

#[derive(Copy, Clone)]
pub enum Type<'expr, 'ident> {
    Named {
        name: TypeName<'ident>,
        span: ast::Span,
    },
    Variant {
        tag: &'ident str,
        arguments: &'expr [Type<'expr, 'ident>],
        span: ast::Span,
    },
    Record {
        fields: &'expr [TypeField<'expr, 'ident>],
        span: ast::Span,
    },
    Wildcard,
    Arrow {
        arguments: &'expr [Type<'expr, 'ident>],
        return_type: &'expr Type<'expr, 'ident>,
        span: ast::Span,
    },
    Union {
        cases: &'expr [Type<'expr, 'ident>],
        span: ast::Span,
    },
}

#[derive(Copy, Clone)]
pub struct TypeField<'expr, 'ident> {
    pub name: &'ident str,
    pub field_type: Type<'expr, 'ident>,
    pub span: ast::Span,
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

impl Debug for Type<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named { name, .. } => name.fmt(f),
            Type::Variant { tag, arguments, .. } => {
                let mut tuple = f.debug_tuple(tag);

                for arg in *arguments {
                    tuple.field(arg);
                }

                tuple.finish()
            }
            Type::Arrow {
                arguments,
                return_type,
                ..
            } => f
                .debug_tuple("func")
                .field(arguments)
                .field(return_type)
                .finish(),
            Type::Wildcard => write!(f, "*"),
            Type::Record { fields, .. } => f
                .debug_map()
                .entries(fields.iter().map(|field| (field.name, field.field_type)))
                .finish(),
            Type::Union { cases, .. } => {
                let mut tuple = f.debug_tuple("union");

                for case in *cases {
                    tuple.field(&case);
                }

                tuple.finish()
            }
        }
    }
}
