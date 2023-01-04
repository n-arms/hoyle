use crate::ast;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

pub type Type<'expr, 'ident> = ast::Type<'expr, Identifier<'ident>>;

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

#[derive(Clone, Debug, Default)]
pub struct TagSource {
    unused_identifier: Rc<Cell<u32>>,
}

impl TagSource {
    pub fn fresh_tag(&mut self, module: u32) -> Tag {
        let identifier = self.unused_identifier.get();
        let tag = Tag { module, identifier };
        self.unused_identifier.set(identifier + 1);
        tag
    }

    pub fn fresh_identifier<'ident>(
        &mut self,
        name: &'ident str,
        module: u32,
    ) -> Identifier<'ident> {
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
