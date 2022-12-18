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
    pub arguments: &'expr [Argument<'expr, 'ident, Id, Ty>],
    pub return_type: Option<Ty>,
    pub body: Expr<'expr, 'ident, Id, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Argument<'expr, 'ident, Id, Ty> {
    pub pattern: Pattern<'expr, 'ident, Id>,
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
    Arrow {
        arguments: &'expr [Type<'expr, 'ident>],
        return_type: &'expr Type<'expr, 'ident>,
        span: Span,
    },
    Record {
        fields: &'expr [TypeField<'expr, 'ident>],
        span: Span,
    },
    Union {
        cases: &'expr [Type<'expr, 'ident>],
        span: Span,
    },
}

#[derive(Copy, Clone)]
pub struct TypeField<'expr, 'ident> {
    pub name: &'ident str,
    pub field_type: Type<'expr, 'ident>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Generic<'ident> {
    pub identifier: &'ident str,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub enum Statement<'expr, 'ident, Id, Ty> {
    Let {
        left_side: Pattern<'expr, 'ident, Id>,
        right_side: Expr<'expr, 'ident, Id, Ty>,
        span: Span,
    },
    Raw(Expr<'expr, 'ident, Id, Ty>, Span),
}

#[derive(Copy, Clone)]
pub enum Pattern<'expr, 'ident, Id> {
    Variable(Id, Span),
    Variant {
        tag: &'ident str,
        arguments: &'expr [Pattern<'expr, 'ident, Id>],
        span: Span,
    },
}

#[derive(Copy, Clone)]
pub struct Block<'expr, 'ident, Id, Ty> {
    pub statements: &'expr [Statement<'expr, 'ident, Id, Ty>],
    pub result: Option<&'expr Expr<'expr, 'ident, Id, Ty>>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Field<'expr, 'ident, Id, Ty> {
    pub name: &'ident str,
    pub value: Expr<'expr, 'ident, Id, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Branch<'expr, 'ident, Id, Ty> {
    pub pattern: Pattern<'expr, 'ident, Id>,
    pub body: Expr<'expr, 'ident, Id, Ty>,
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
        tag: &'ident str,
        arguments: &'expr [Expr<'expr, 'ident, Id, Ty>],
        span: Span,
    },
    Record {
        fields: &'expr [Field<'expr, 'ident, Id, Ty>],
        span: Span,
    },
    Block(Block<'expr, 'ident, Id, Ty>),
    Annotated {
        expr: &'expr Expr<'expr, 'ident, Id, Ty>,
        annotation: Ty,
        span: Span,
    },
    Case {
        predicate: &'expr Expr<'expr, 'ident, Id, Ty>,
        branches: &'expr [Branch<'expr, 'ident, Id, Ty>],
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
            | Expr::Record { span, .. }
            | Expr::Case { span, .. }
            | Expr::Block(Block { span, .. }) => *span,
        }
    }
}

impl<Id> Pattern<'_, '_, Id> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Pattern::Variable(_, span) | Pattern::Variant { span, .. } => *span,
        }
    }
}

impl Type<'_, '_> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Type::Named(_, span)
            | Type::Variant { span, .. }
            | Type::Arrow { span, .. }
            | Type::Union { span, .. }
            | Type::Record { span, .. } => *span,
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

impl<Id: Debug, Ty: Debug> Debug for Argument<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.pattern, self.type_annotation)
    }
}

impl Debug for Generic<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.identifier.fmt(f)
    }
}

impl<Id: Debug> Debug for Pattern<'_, '_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Variable(variable, _) => variable.fmt(f),
            Pattern::Variant { tag, arguments, .. } => {
                let mut tuple = f.debug_tuple(tag);

                for arg in *arguments {
                    tuple.field(&arg);
                }

                tuple.finish()
            }
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
            Expr::Variant { tag, arguments, .. } => {
                let mut tuple = f.debug_tuple(tag);
                for arg in *arguments {
                    tuple.field(arg);
                }
                tuple.finish()
            }
            Expr::Record { fields, .. } => f
                .debug_map()
                .entries(fields.iter().map(|field| (field.name, &field.value)))
                .finish(),
            Expr::Block(block) => block.fmt(f),
            Expr::Annotated {
                expr, annotation, ..
            } => {
                write!(f, "{:?}: {:?}", expr, annotation)
            }
            Expr::Case {
                predicate,
                branches,
                ..
            } => f
                .debug_tuple("case")
                .field(predicate)
                .field(branches)
                .finish(),
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
            Type::Arrow {
                arguments,
                return_type,
                ..
            } => f
                .debug_tuple("func")
                .field(arguments)
                .field(return_type)
                .finish(),
            Type::Record { fields, .. } => f
                .debug_map()
                .entries(fields.iter().map(|field| (field.name, &field.field_type)))
                .finish(),
            Type::Union { cases, .. } => {
                let mut tuple = f.debug_tuple("union");

                for case in *cases {
                    tuple.field(case);
                }

                tuple.finish()
            }
        }
    }
}

impl<Id: Debug, Ty: Debug> Debug for Field<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.value)
    }
}

impl<Id: Debug, Ty: Debug> Debug for Branch<'_, '_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.pattern.fmt(f)?;
        write!(f, " => ")?;
        self.body.fmt(f)
    }
}
