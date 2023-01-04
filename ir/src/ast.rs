use arena_alloc::General;

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
pub struct Program<'expr, Id, Ty> {
    pub definitions: &'expr [Definition<'expr, Id, Ty>],
}

#[derive(Copy, Clone)]
pub enum Definition<'expr, Id, Ty> {
    Function {
        name: Id,
        generics: &'expr [Generic<Ty>],
        arguments: &'expr [Argument<'expr, Id, Ty>],
        return_type: Option<Type<'expr, Ty>>,
        body: Expr<'expr, Id, Ty>,
        span: Span,
    },
    Struct {
        name: Id,
        fields: &'expr [FieldDefinition<'expr, Id, Ty>],
        span: Span,
    },
}

#[derive(Copy, Clone)]
pub struct FieldDefinition<'expr, Id, Ty> {
    pub name: Id,
    pub field_type: Type<'expr, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Argument<'expr, Id, Ty> {
    pub pattern: Pattern<'expr, Id>,
    pub type_annotation: Type<'expr, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub enum Type<'expr, Ty> {
    Named {
        name: Ty,
        span: Option<Span>,
    },
    Arrow {
        arguments: &'expr [Type<'expr, Ty>],
        return_type: &'expr Type<'expr, Ty>,
        span: Option<Span>,
    },
}

#[derive(Copy, Clone)]
pub struct Generic<Ty> {
    pub identifier: Ty,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub enum Statement<'expr, Id, Ty> {
    Let {
        left_side: Pattern<'expr, Id>,
        right_side: Expr<'expr, Id, Ty>,
        span: Span,
    },
    Raw(Expr<'expr, Id, Ty>, Span),
}

#[derive(Copy, Clone)]
pub struct PatternField<'expr, Id> {
    pub name: Id,
    pub pattern: Pattern<'expr, Id>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub enum Pattern<'expr, Id> {
    Variable(Id, Span),
    Struct {
        name: Id,
        fields: &'expr [PatternField<'expr, Id>],
        span: Span,
    },
}

#[derive(Copy, Clone)]
pub struct Block<'expr, Id, Ty> {
    pub statements: &'expr [Statement<'expr, Id, Ty>],
    pub result: Option<&'expr Expr<'expr, Id, Ty>>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Field<'expr, Id, Ty> {
    pub name: Id,
    pub value: Expr<'expr, Id, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone)]
pub struct Branch<'expr, Id, Ty> {
    pub pattern: Pattern<'expr, Id>,
    pub body: Expr<'expr, Id, Ty>,
    pub span: Span,
}

#[derive(Copy, Clone)]
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
    StructLiteral {
        name: Id,
        fields: &'expr [Field<'expr, Id, Ty>],
        span: Span,
    },
    Block(Block<'expr, Id, Ty>),
    Annotated {
        expr: &'expr Expr<'expr, Id, Ty>,
        annotation: Type<'expr, Ty>,
        span: Span,
    },
    Case {
        predicate: &'expr Expr<'expr, Id, Ty>,
        branches: &'expr [Branch<'expr, Id, Ty>],
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

impl<'old> Literal<'old> {
    #[must_use]
    pub fn realloc<'new>(&self, alloc: &General<'new>) -> Literal<'new> {
        match self {
            Literal::Integer(int) => Literal::Integer(alloc.alloc_str(int)),
        }
    }
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
            | Expr::StructLiteral { span, .. }
            | Expr::Case { span, .. }
            | Expr::Block(Block { span, .. }) => *span,
        }
    }
}

impl<Id> Pattern<'_, Id> {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Pattern::Variable(_, span) | Pattern::Struct { span, .. } => *span,
        }
    }
}

impl<Ty> Type<'_, Ty> {
    #[must_use]
    pub const fn span(&self) -> Option<Span> {
        match self {
            Type::Named { span, .. } | Type::Arrow { span, .. } => *span,
        }
    }
}

impl<Id, Ty> FieldDefinition<'_, Id, Ty> {
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

impl<Id: Debug, Ty: Debug> Debug for Program<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.definitions).finish()
    }
}

impl<Id: Debug, Ty: Debug> Debug for Definition<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Definition::Function {
                name,
                generics,
                arguments,
                return_type,
                body,
                ..
            } => {
                let mut tuple = f.debug_tuple("func");
                tuple.field(name).field(generics).field(arguments);

                if let Some(return_type) = return_type {
                    tuple.field(&return_type);
                }

                tuple.field(body).finish()
            }
            Definition::Struct { name, fields, .. } => {
                let mut r#struct = f.debug_struct(&format!("{name:?}"));

                for field in *fields {
                    r#struct.field(&format!("{:?}", field.name), &field.field_type);
                }

                r#struct.finish()
            }
        }
    }
}

impl<Id: Debug, Ty: Debug> Debug for Argument<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.pattern, self.type_annotation)
    }
}

impl<Ty: Debug> Debug for Generic<Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.identifier.fmt(f)
    }
}

impl<Id: Debug> Debug for Pattern<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Variable(variable, _) => variable.fmt(f),
            Pattern::Struct { fields, .. } => f
                .debug_map()
                .entries(fields.iter().map(|f| (&f.name, &f.pattern)))
                .finish(),
        }
    }
}

impl<Id: Debug> Debug for PatternField<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)?;
        write!(f, ": ")?;
        self.pattern.fmt(f)
    }
}

impl<Id: Debug, Ty: Debug> Debug for Statement<'_, Id, Ty> {
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

impl<Id: Debug, Ty: Debug> Debug for Block<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut block = f.debug_list();

        block.entries(self.statements);

        if let Some(result) = self.result {
            block.entry(result);
        }

        block.finish()
    }
}

impl<Id: Debug, Ty: Debug> Debug for Expr<'_, Id, Ty> {
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
                let mut tuple = f.debug_tuple(&format!("{operator:?}"));
                for arg in *arguments {
                    tuple.field(arg);
                }
                tuple.finish()
            }
            Expr::StructLiteral { fields, .. } => f
                .debug_map()
                .entries(fields.iter().map(|field| (&field.name, &field.value)))
                .finish(),
            Expr::Block(block) => block.fmt(f),
            Expr::Annotated {
                expr, annotation, ..
            } => {
                write!(f, "{expr:?}: {annotation:?}")
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

impl<Ty: Debug> Debug for Type<'_, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named { name, .. } => name.fmt(f),
            Type::Arrow {
                arguments,
                return_type,
                ..
            } => f
                .debug_tuple("func")
                .field(arguments)
                .field(return_type)
                .finish(),
        }
    }
}

impl<Id: Debug, Ty: Debug> Debug for Field<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.name, self.value)
    }
}

impl<Id: Debug, Ty: Debug> Debug for Branch<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.pattern.fmt(f)?;
        write!(f, " => ")?;
        self.body.fmt(f)
    }
}

impl<Id: Debug, Ty: Debug> Debug for FieldDefinition<'_, Id, Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.name, self.field_type)
    }
}
