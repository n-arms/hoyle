use crate::ast;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};

pub type Type<'expr, 'ident> = ast::Type<'expr, Identifier<'ident>>;

#[derive(Copy, Clone)]
pub struct Primitives<'ident> {
    pub int: Identifier<'ident>,
    pub bool: Identifier<'ident>,
}

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

impl<'ident> Identifier<'ident> {
    #[must_use]
    pub const fn new(tag: Tag, name: &'ident str) -> Self {
        Self { tag, name }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StructDefinition<'expr, 'ident> {
    pub name: Identifier<'ident>,
    pub fields: &'expr [FieldDefinition<'expr, 'ident>],
}

impl<'expr, 'ident> StructDefinition<'expr, 'ident> {
    #[must_use]
    pub fn find_field(&self, name: Identifier<'ident>) -> Option<FieldDefinition<'expr, 'ident>> {
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

#[derive(Copy, Clone, Debug)]
pub struct LocalTagSource<'t> {
    module: u32,
    tags: &'t TagSource,
}

impl<'t> LocalTagSource<'t> {
    pub const fn new(module: u32, tags: &'t TagSource) -> Self {
        Self { module, tags }
    }

    #[must_use]
    pub fn fresh_tag(&self) -> Tag {
        self.tags.fresh_tag(self.module)
    }

    #[must_use]
    pub fn fresh_identifier<'ident>(&self, name: &'ident str) -> Identifier<'ident> {
        self.tags.fresh_identifier(name, self.module)
    }
}

#[derive(Debug, Default)]
pub struct TagSource {
    unused_identifier: Cell<u32>,
}

impl TagSource {
    pub fn fresh_tag(&self, module: u32) -> Tag {
        let identifier = self.unused_identifier.get();
        let tag = Tag { module, identifier };
        self.unused_identifier.set(identifier + 1);
        tag
    }

    pub fn fresh_identifier<'ident>(&self, name: &'ident str, module: u32) -> Identifier<'ident> {
        Identifier {
            tag: self.fresh_tag(module),
            name,
        }
    }
}

pub type Program<'expr, 'ident> = ast::Program<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Definition<'expr, 'ident> = ast::Definition<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Generic<'ident> = ast::Generic<Identifier<'ident>>;

pub type FieldDefinition<'expr, 'ident> =
    ast::FieldDefinition<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Argument<'expr, 'ident> = ast::Argument<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Statement<'expr, 'ident> = ast::Statement<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, Identifier<'ident>>;

pub type Block<'expr, 'ident> = ast::Block<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Field<'expr, 'ident> = ast::Field<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Branch<'expr, 'ident> = ast::Branch<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type Expr<'expr, 'ident> = ast::Expr<'expr, Identifier<'ident>, Identifier<'ident>>;

pub type PatternField<'expr, 'ident> = ast::PatternField<'expr, Identifier<'ident>>;

impl Debug for Identifier<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}::{}", self.tag, self.name)
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}.{}", self.module, self.identifier)
    }
}
