use smartstring::{LazyCompact, SmartString};

#[derive(Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone)]
pub struct Program<'expr> {
    pub definitions: &'expr [Definition<'expr>],
}

#[derive(Clone)]
pub enum Definition<'expr> {
    Function(FunctionDefinition<'expr>),
    Struct(StructDefinition<'expr>),
}

#[derive(Clone)]
pub struct FunctionDefinition<'expr> {
    pub name: Identifier,
    pub generics: &'expr [GenericDefinition],
    pub arguments: &'expr [ArgumentDefinition<'expr>],
    pub return_type: Type<'expr>,
    pub body: Expr<'expr>,
    pub span: Span,
}

#[derive(Clone)]
pub struct StructDefinition<'expr> {
    pub name: Identifier,
    pub fields: &'expr [FieldDefinition<'expr>],
    pub span: Span,
}

#[derive(Clone)]
pub struct Identifier {
    pub name: SmartString<LazyCompact>,
}

#[derive(Clone)]
pub struct FieldDefinition<'expr> {
    pub field: Identifier,
    pub r#type: Type<'expr>,
    pub span: Span,
}

#[derive(Clone)]
pub struct ArgumentDefinition<'expr> {
    pub pattern: Pattern<'expr>,
    pub r#type: Type<'expr>,
    pub span: Span,
}

#[derive(Clone)]
pub struct GenericDefinition {
    pub name: Identifier,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct PatternField<'expr> {
    pub name: Identifier,
    pub pattern: Pattern<'expr>,
    pub span: Span,
}

#[derive(Clone)]
pub enum Literal {
    Boolean(bool),
    Number(f64),
}

#[derive(Clone)]
pub enum Operation {
    Add,
}

#[derive(Clone)]
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
        funcion: &'expr Expr<'expr>,
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

#[derive(Clone)]
pub struct Field<'expr> {
    pub name: Identifier,
    pub value: Expr<'expr>,
}

#[derive(Clone)]
pub struct Block<'expr> {
    pub statements: &'expr [Statement<'expr>],
}

#[derive(Clone)]
pub enum Statement<'expr> {
    Raw(Expr<'expr>),
    Let {
        pattern: Pattern<'expr>,
        value: Expr<'expr>,
        span: Span,
    },
}

#[derive(Clone)]
pub struct Branch<'expr> {
    pub pattern: Pattern<'expr>,
    pub body: Expr<'expr>,
    pub span: Span,
}

#[derive(Clone)]
pub enum Type<'expr> {
    Named(Identifier),
    Function {
        arguments: &'expr [Type<'expr>],
        span: Span,
    },
    Application {
        main: Identifier,
        arguments: &'expr [Type<'expr>],
    },
}
