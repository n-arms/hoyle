use crate::token;
use std::fmt::{Debug, Formatter};
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

#[derive(Copy, Clone, Default)]
pub struct Program<'expr, 'ident, Id, Ty> {
    pub definitions: &'expr [Definition<'expr, 'ident, Id, Ty>],
}

#[derive(Copy, Clone)]
pub struct Definition<'expr, 'ident, Id, Ty> {
    pub name: &'ident str,
    pub generics: &'expr [Generic<'ident>],
    pub arguments: &'expr [Argument<'expr, Id, Ty>],
    pub return_type: Option<Ty>,
    pub body: Expr<'expr, 'ident, Id, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Argument<'expr, Id, Ty> {
    pub pattern: Pattern<'expr, Id>,
    pub type_annotation: Ty,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub enum Type<'expr, 'ident> {
    Named(&'ident str, Span),
    Variant {
        tag: &'ident str,
        arguments: &'expr [Type<'expr, 'ident>],
        span: Span,
    },
    Tuple(&'expr [Type<'expr, 'ident>], Span),
}

#[derive(Copy, Clone)]
pub struct Generic<'ident> {
    pub identifier: &'ident str,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub enum Statement<'expr, 'ident, Id, Ty> {
    Let {
        left_side: Pattern<'expr, Id>,
        right_side: Expr<'expr, 'ident, Id, Ty>,
        span: Span,
    },
    Raw(Expr<'expr, 'ident, Id, Ty>, Span),
}

#[derive(Copy, Clone)]
pub enum Pattern<'expr, Id> {
    Variable(Id, Span),
    Tuple(&'expr [Pattern<'expr, Id>], Span),
}

#[derive(Copy, Clone)]
pub struct Block<'expr, 'ident, Id, Ty> {
    pub statements: &'expr [Statement<'expr, 'ident, Id, Ty>],
    pub result: Option<&'expr Expr<'expr, 'ident, Id, Ty>>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub enum Expr<'expr, 'ident, Id, Ty> {
    Variable(Id, Span),
    Literal(Literal<'expr>, Span),
    Call {
        function: &'expr Expr<'expr, 'ident, Id, Ty>,
        arguments: &'expr [Expr<'expr, 'ident, Id, Ty>],
        span: Span,
    },
    Operation {
        operator: Operator,
        arguments: &'expr [Expr<'expr, 'ident, Id, Ty>],
        span: Span,
    },
    Variant {
        variant: &'ident str,
        arguments: &'expr [Expr<'expr, 'ident, Id, Ty>],
        span: Span,
    },
    Block(Block<'expr, 'ident, Id, Ty>),
    Annotated {
        expr: &'expr Expr<'expr, 'ident, Id, Ty>,
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

impl<Id, Ty> Expr<'_, '_, Id, Ty> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Expr::Variable(_, span)
            | Expr::Literal(_, span)
            | Expr::Call { span, .. }
            | Expr::Operation { span, .. }
            | Expr::Annotated { span, .. }
            | Expr::Variant { span, .. }
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
            Type::Named(_, span) | Type::Variant { span, .. } | Type::Tuple(_, span) => *span,
        }
    }
}

impl<Id: Debug, Ty: Debug> Debug for Program<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.definitions).finish()
    }
}

impl<Id: Debug, Ty: Debug> Debug for Definition<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("func");
        tuple
            .field(&self.name)
            .field(&self.generics)
            .field(&self.arguments);

        if let Some(return_type) = &self.return_type {
            tuple.field(&return_type);
        }

        tuple.field(&self.body).finish()
    }
}

impl<Id: Debug, Ty: Debug> Debug for Argument<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.pattern, self.type_annotation)
    }
}

impl Debug for Generic<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.identifier.fmt(f)
    }
}

impl<Id: Debug> Debug for Pattern<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Variable(variable, _) => variable.fmt(f),
            Pattern::Tuple(arguments, _) => f.debug_list().entries(*arguments).finish(),
        }
    }
}

impl<Id: Debug, Ty: Debug> Debug for Statement<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let {
                left_side,
                right_side,
                ..
            } => f
                .debug_tuple("let")
                .field(left_side)
                .field(right_side)
                .finish(),
            Statement::Raw(expr, _) => f.debug_tuple("raw").field(expr).finish(),
        }
    }
}

impl<Id: Debug, Ty: Debug> Debug for Block<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut block = f.debug_list();

        block.entries(self.statements);

        if let Some(result) = self.result {
            block.entry(result);
        }

        block.finish()
    }
}

impl<Id: Debug, Ty: Debug> Debug for Expr<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Variable(variable, _) => variable.fmt(f),
            Expr::Literal(literal, _) => literal.fmt(f),
            Expr::Call {
                function,
                arguments,
                ..
            } => f.debug_list().entry(function).entries(*arguments).finish(),
            Expr::Operation {
                operator,
                arguments,
                ..
            } => {
                let mut tuple = f.debug_tuple(&format!("{:?}", operator));
                for arg in *arguments {
                    tuple.field(arg);
                }
                tuple.finish()
            }
            Expr::Variant {
                variant, arguments, ..
            } => {
                let mut tuple = f.debug_tuple(variant);
                for arg in *arguments {
                    tuple.field(arg);
                }
                tuple.finish()
            }
            Expr::Block(block) => block.fmt(f),
            Expr::Annotated {
                expr, annotation, ..
            } => {
                write!(f, "{:?}: {:?}", expr, annotation)
            }
        }
    }
}

impl Debug for Type<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named(name, _) => name.fmt(f),
            Type::Variant { tag, arguments, .. } => {
                let mut tuple = f.debug_tuple(tag);

                for arg in *arguments {
                    tuple.field(arg);
                }

                tuple.finish()
            }
            Type::Tuple(_, _) => todo!(),
        }
    }
}
