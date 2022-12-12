use crate::token;
use std::ops::Range;

// change the types of identifiers (canonicalize import paths)
// change the types of generics
//    - Generic [ &str, Vec<Constraint<SourceType | CanonType | Type>> ]
// change the types of identifiers
//    - &str -> Qualified [ &str, ImportPath, CanonType | Type ]
// change the types of types
//    - SourceType -> CanonType -> Type

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Program<'expr, 'ident, Id, Ty> {
    pub definitions: &'expr [Definition<'expr, 'ident, Id, Ty>],
}

#[derive(Copy, Clone, Debug)]
pub struct Definition<'expr, 'ident, Id, Ty> {
    pub name: &'ident str,
    pub generics: &'expr [Generic<'ident>],
    pub arguments: &'expr [Argument<'expr, Id, Ty>],
    pub return_type: Option<Ty>,
    pub body: Expr<'expr, Id, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub struct Argument<'expr, Id, Ty> {
    pub pattern: Pattern<'expr, Id>,
    pub type_annotation: Ty,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Type<'expr, 'ident> {
    Named(&'ident str, Span),
    Tuple(&'expr [Type<'expr, 'ident>], Span),
}

#[derive(Copy, Clone, Debug)]
pub struct Generic<'ident> {
    pub identifier: &'ident str,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Statement<'expr, Id, Ty> {
    Let {
        left_side: Pattern<'expr, Id>,
        right_side: Expr<'expr, Id, Ty>,
        span: Span,
    },
    Raw(Expr<'expr, Id, Ty>, Span),
}

#[derive(Copy, Clone, Debug)]
pub enum Pattern<'expr, Id> {
    Variable(Id, Span),
    Tuple(&'expr [Pattern<'expr, Id>], Span),
}

#[derive(Copy, Clone, Debug)]
pub struct Block<'expr, Id, Ty> {
    pub statements: &'expr [Statement<'expr, Id, Ty>],
    pub result: Option<&'expr Expr<'expr, Id, Ty>>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Expr<'expr, Id, Ty> {
    Variable(Id, Span),
    Literal(Literal<'expr>, Span),
    Call {
        function: &'expr Expr<'expr, Id, Ty>,
        arguments: &'expr [Expr<'expr, Id, Ty>],
        span: Span,
    },
    Operation {
        operator: Operator,
        arguments: &'expr [Expr<'expr, Id, Ty>],
        span: Span,
    },
    Block(Block<'expr, Id, Ty>),
    Annotated {
        expr: &'expr Expr<'expr, Id, Ty>,
        annotation: Ty,
        span: Span,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum Literal<'expr> {
    Integer(&'expr str),
}

#[derive(Copy, Clone, Debug)]
pub enum Operator {
    Add,
    Sub,
    Times,
    Div,
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<token::Span<'_>> for Span {
    fn from(span: token::Span) -> Self {
        Self::new(span.offset, span.data.len())
    }
}

impl Span {
    #[must_use]
    pub const fn new(start: usize, len: usize) -> Self {
        Self {
            start,
            end: start + len,
        }
    }

    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self::from(self.start.min(other.start)..self.end.max(other.end))
    }
}

impl<Id, Ty> Expr<'_, Id, Ty> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Expr::Variable(_, span)
            | Expr::Literal(_, span)
            | Expr::Call { span, .. }
            | Expr::Operation { span, .. }
            | Expr::Annotated { span, .. }
            | Expr::Block(Block { span, .. }) => *span,
        }
    }
}

impl<Id> Pattern<'_, Id> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Pattern::Variable(_, span) | Pattern::Tuple(_, span) => *span,
        }
    }
}

impl Type<'_, '_> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Type::Named(_, span) | Type::Tuple(_, span) => *span,
        }
    }
}
