use crate::token;
use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Program<'expr, ID> {
    pub definitions: &'expr [Definition<'expr, ID>],
}

#[derive(Copy, Clone, Debug)]
pub struct Definition<'expr, ID> {
    pub name: ID,
    pub generics: &'expr [Generic<ID>],
    pub arguments: &'expr [Argument<'expr, ID>],
    pub return_type: Option<Type<'expr, ID>>,
    pub body: Expr<'expr, ID>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub struct Argument<'expr, ID> {
    pub pattern: Pattern<'expr, ID>,
    pub type_annotation: Type<'expr, ID>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Type<'expr, ID> {
    Named(ID, Span),
    Tuple(&'expr [Type<'expr, ID>], Span),
}

#[derive(Copy, Clone, Debug)]
pub struct Generic<ID> {
    pub identifier: ID,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Statement<'expr, ID> {
    Let {
        left_side: Pattern<'expr, ID>,
        right_side: Expr<'expr, ID>,
        span: Span,
    },
    Raw(Expr<'expr, ID>, Span),
}

#[derive(Copy, Clone, Debug)]
pub enum Pattern<'expr, ID> {
    Variable(ID, Span),
    Tuple(&'expr [Pattern<'expr, ID>], Span),
}

#[derive(Copy, Clone, Debug)]
pub struct Block<'expr, ID> {
    pub statements: &'expr [Statement<'expr, ID>],
    pub result: Option<&'expr Expr<'expr, ID>>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Expr<'expr, ID> {
    Variable(ID, Span),
    Literal(Literal<'expr>, Span),
    Call {
        function: &'expr Expr<'expr, ID>,
        arguments: &'expr [Expr<'expr, ID>],
        span: Span,
    },
    Operation {
        operator: Operator,
        arguments: &'expr [Expr<'expr, ID>],
        span: Span,
    },
    Block(Block<'expr, ID>),
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

impl<ID> Expr<'_, ID> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Expr::Variable(_, span)
            | Expr::Literal(_, span)
            | Expr::Call { span, .. }
            | Expr::Operation { span, .. }
            | Expr::Block(Block { span, .. }) => *span,
        }
    }
}

impl<ID> Pattern<'_, ID> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Pattern::Variable(_, span) | Pattern::Tuple(_, span) => *span,
        }
    }
}

impl<ID> Type<'_, ID> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Type::Named(_, span) | Type::Tuple(_, span) => *span,
        }
    }
}
