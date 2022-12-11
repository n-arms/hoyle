use crate::token;
use std::ops::Range;

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Program<'ident, 'expr> {
    pub definitions: &'expr [Definition<'ident, 'expr>],
}

#[derive(Copy, Clone, Debug)]
pub struct Definition<'ident, 'expr> {
    pub name: &'ident str,
    pub generics: &'expr [Generic<'ident>],
    pub arguments: &'expr [Argument<'ident, 'expr>],
    pub return_type: Option<Type<'ident, 'expr>>,
    pub body: Expr<'ident, 'expr>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub struct Argument<'ident, 'expr> {
    pub pattern: Pattern<'ident, 'expr>,
    pub type_annotation: Type<'ident, 'expr>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Type<'ident, 'expr> {
    Named(&'ident str, Span),
    Tuple(&'expr [Type<'ident, 'expr>], Span),
}

#[derive(Copy, Clone, Debug)]
pub struct Generic<'ident> {
    pub identifier: &'ident str,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Statement<'ident, 'expr> {
    Let {
        left_side: Pattern<'ident, 'expr>,
        right_side: Expr<'ident, 'expr>,
        span: Span,
    },
    Raw(Expr<'ident, 'expr>, Span),
}

#[derive(Copy, Clone, Debug)]
pub enum Pattern<'ident, 'expr> {
    Variable(&'ident str, Span),
    Tuple(&'expr [Pattern<'ident, 'expr>], Span),
}

#[derive(Copy, Clone, Debug)]
pub struct Block<'ident, 'expr> {
    pub statements: &'expr [Statement<'ident, 'expr>],
    pub result: Option<&'expr Expr<'ident, 'expr>>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Expr<'ident, 'expr> {
    Variable(&'ident str, Span),
    Literal(Literal<'expr>, Span),
    Call {
        function: &'expr Expr<'ident, 'expr>,
        arguments: &'expr [Expr<'ident, 'expr>],
        span: Span,
    },
    Operation {
        operator: Operator,
        arguments: &'expr [Expr<'ident, 'expr>],
        span: Span,
    },
    Block(Block<'ident, 'expr>, Span),
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
    #[must_use] pub const fn new(start: usize, len: usize) -> Self {
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

impl Expr<'_, '_> {
    #[must_use] pub fn span(&self) -> Span {
        match self {
            Expr::Variable(_, span)
            | Expr::Literal(_, span)
            | Expr::Call { span, .. }
            | Expr::Operation { span, .. }
            | Expr::Block(_, span) => *span,
        }
    }
}

impl Pattern<'_, '_> {
    #[must_use] pub fn span(&self) -> Span {
        match self {
            Pattern::Variable(_, span) | Pattern::Tuple(_, span) => *span,
        }
    }
}

impl Type<'_, '_> {
    #[must_use] pub fn span(&self) -> Span {
        match self {
            Type::Named(_, span) | Type::Tuple(_, span) => *span,
        }
    }
}
