use smartstring::{LazyCompact, SmartString};

use crate::token;

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub name: SmartString<LazyCompact>,
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
        operator: Type<'expr>,
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

impl Type<'_> {
    pub fn span(&self) -> Span {
        match self {
            Type::Named { span, .. } => *span,
            Type::Function { span, .. } => *span,
            Type::Application { span, .. } => *span,
        }
    }
}

impl Span {
    pub fn union(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<token::Span<'_>> for Span {
    fn from(value: token::Span<'_>) -> Self {
        Self {
            start: value.offset,
            end: value.offset + value.data.len(),
        }
    }
}

impl Pattern<'_> {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Variable { span, .. } => *span,
            Pattern::Struct { span, .. } => *span,
        }
    }
}

impl Expr<'_> {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal { span, .. } => *span,
            Expr::Variable { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Operation { span, .. } => *span,
            Expr::StructLiteral { span, .. } => *span,
            Expr::Block(block) => block.span,
            Expr::Annotated { span, .. } => *span,
            Expr::Case { span, .. } => *span,
        }
    }
}
