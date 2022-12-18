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
    Tuple(&'expr [Type<'expr, 'ident>], ast::Span),
    Wildcard,
    Arrow {
        arguments: &'expr [Type<'expr, 'ident>],
        return_type: &'expr Type<'expr, 'ident>,
        span: ast::Span,
    },
}

pub type Program<'expr, 'ident> =
    ast::Program<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, Identifier<'expr, 'ident>>;

pub type Block<'expr, 'ident> =
    ast::Block<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Field<'expr, 'ident> =
    ast::Field<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Expr<'expr, 'ident> =
    ast::Expr<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

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
            IdentifierSource::Local => write!(f, "local"),
            IdentifierSource::Global(path) => path.fmt(f),
        }
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Path::Current => write!(f, "current"),
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
            Type::Tuple(_, _) => todo!(),
        }
    }
}
