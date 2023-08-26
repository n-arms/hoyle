use smartstring::{LazyCompact, SmartString};

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
}

#[derive(Clone)]
pub struct StructDefinition<'expr> {
    pub name: Identifier,
    pub fields: &'expr [FieldDefinition<'expr>],
}

#[derive(Clone)]
pub struct Tag {
    pub module: u32,
    pub key: u32,
}

#[derive(Clone)]
pub struct Identifier {
    pub name: SmartString<LazyCompact>,
    pub tag: Tag,
}

#[derive(Clone)]
pub struct FieldDefinition<'expr> {
    pub field: Identifier,
    pub r#type: Type<'expr>,
}

#[derive(Clone)]
pub struct ArgumentDefinition<'expr> {
    pub pattern: Pattern<'expr>,
    pub r#type: Type<'expr>,
}

#[derive(Clone)]
pub struct GenericDefinition {
    pub name: Identifier,
}

#[derive(Clone)]
pub enum Pattern<'expr> {
    Variable {
        name: Identifier,
        r#type: Type<'expr>,
    },
    Struct {
        name: Identifier,
        fields: &'expr [PatternField<'expr>],
        r#type: Type<'expr>,
    },
}

#[derive(Clone)]
pub struct PatternField<'expr> {
    pub name: Identifier,
    pub pattern: Pattern<'expr>,
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
    },
    Variable {
        identifier: Identifier,
        /// the monomorphized type of the variable
        r#type: Type<'expr>,
        /// the types used to specialize the variable
        specialized_to: &'expr [Type<'expr>],
    },
    Call {
        funcion: &'expr Expr<'expr>,
        arguments: &'expr [Expr<'expr>],
        r#type: Type<'expr>,
    },
    Operation {
        operator: Type<'expr>,
        arguments: &'expr [Expr<'expr>],
    },
    StructLiteral {
        name: Identifier,
        fields: &'expr [Field<'expr>],
        r#type: Type<'expr>,
    },
    Block(Block<'expr>),
    Annotated {
        expr: &'expr Expr<'expr>,
        annotation: Type<'expr>,
    },
    Case {
        predicate: &'expr Expr<'expr>,
        branches: &'expr [Branch<'expr>],
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
    },
}

#[derive(Clone)]
pub struct Branch<'expr> {
    pub pattern: Pattern<'expr>,
    pub body: Expr<'expr>,
}

#[derive(Clone)]
pub enum Type<'expr> {
    Named(Identifier),
    Generic(Identifier),
    Function {
        arguments: &'expr [Type<'expr>],
    },
    Application {
        main: Identifier,
        arguments: &'expr [Type<'expr>],
    },
}
