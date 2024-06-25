use crate::source::{self, Span};
use smartstring::{LazyCompact, SmartString};
use std::cell::Cell;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IdentifierSource {
    Local,
    Global(Path),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Path {
    Builtin,
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
    pub fn fresh_identifier(&self, name: SmartString<LazyCompact>) -> Identifier {
        self.tags.fresh_identifier(name, self.module)
    }
}

#[derive(Debug, Default)]
pub struct TagSource {
    unused_key: Cell<u32>,
}

impl TagSource {
    pub fn fresh_tag(&self, module: u32) -> Tag {
        let key = self.unused_key.get();
        let tag = Tag { module, key };
        self.unused_key.set(key + 1);
        tag
    }

    pub fn fresh_identifier(&self, name: SmartString<LazyCompact>, module: u32) -> Identifier {
        Identifier {
            tag: self.fresh_tag(module),
            name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Primitives {
    pub integer: Identifier,
    pub boolean: Identifier,
}

#[derive(Debug, Clone)]
pub struct Program<'expr> {
    pub definitions: &'expr [Definition<'expr>],
}

#[derive(Debug, Clone)]
pub enum Definition<'expr> {
    Function(FunctionDefinition<'expr>),
    Struct(StructDefinition<'expr>),
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition<'expr> {
    pub name: Identifier,
    pub generics: &'expr [GenericDefinition],
    pub arguments: &'expr [ArgumentDefinition<'expr>],
    pub return_type: Type<'expr>,
    pub body: Expr<'expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDefinition<'expr> {
    pub name: Identifier,
    pub fields: &'expr [FieldDefinition<'expr>],
    pub span: Span,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag {
    pub module: u32,
    pub key: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    pub name: SmartString<LazyCompact>,
    pub tag: Tag,
}

#[derive(Debug, Clone)]
pub struct FieldDefinition<'expr> {
    pub name: Identifier,
    pub r#type: Type<'expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ArgumentDefinition<'expr> {
    pub pattern: Pattern<'expr>,
    pub r#type: Type<'expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GenericDefinition {
    pub name: Identifier,
}

#[derive(Debug, Clone)]
pub enum Pattern<'expr> {
    Variable {
        name: Identifier,
        span: Span,
    },
    Struct {
        name: Identifier,
        fields: &'expr [PatternField<'expr>],
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct PatternField<'expr> {
    pub name: Identifier,
    pub pattern: Pattern<'expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
}

#[derive(Debug, Clone)]
pub enum Expr<'expr> {
    Literal {
        literal: Literal,
        span: Span,
    },
    Variable {
        identifier: Identifier,
        span: Span,
    },
    Call {
        function: &'expr Expr<'expr>,
        arguments: &'expr [Expr<'expr>],
        span: Span,
    },
    Operation {
        operator: Operation,
        arguments: &'expr [Expr<'expr>],
        span: Span,
    },
    StructLiteral {
        name: Identifier,
        fields: &'expr [Field<'expr>],
        span: Span,
    },
    Block(Block<'expr>),
    Annotated {
        expr: &'expr Expr<'expr>,
        annotation: Type<'expr>,
        span: Span,
    },
    Case {
        predicate: &'expr Expr<'expr>,
        branches: &'expr [Branch<'expr>],
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct Field<'expr> {
    pub name: Identifier,
    pub value: Expr<'expr>,
}

#[derive(Debug, Clone)]
pub struct Block<'expr> {
    pub statements: &'expr [Statement<'expr>],
    pub result: Option<&'expr Expr<'expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement<'expr> {
    Raw(Expr<'expr>),
    Let {
        pattern: Pattern<'expr>,
        value: Expr<'expr>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct Branch<'expr> {
    pub pattern: Pattern<'expr>,
    pub body: Expr<'expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Type<'expr> {
    Named {
        name: Identifier,
        span: Span,
    },
    Generic {
        name: Identifier,
        span: Span,
    },
    Function {
        arguments: &'expr [Type<'expr>],
        return_type: &'expr Type<'expr>,
        span: Span,
    },
    Application {
        main: Identifier,
        arguments: &'expr [Type<'expr>],
        span: Span,
    },
}

impl Identifier {
    pub fn new(tag: Tag, identifier: source::Identifier) -> Self {
        Self {
            name: identifier.name,
            tag,
        }
    }
}
