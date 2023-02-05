use crate::qualified::Tag;

#[derive(Copy, Clone, Debug)]
pub struct Program<'expr> {
    pub functions: &'expr [FunctionDefinition<'expr>],
    pub structs: &'expr [StructDefinition<'expr>],
}

#[derive(Copy, Clone, Debug)]
pub struct FunctionDefinition<'expr> {
    pub label: Tag,
    pub arguments: &'expr [Argument<'expr>],
    pub body: Block<'expr>,
}

#[derive(Copy, Clone, Debug)]
pub struct Argument<'expr> {
    pub name: Tag,
    pub r#type: Type<'expr>,
}

#[derive(Copy, Clone, Debug)]
pub struct Block<'expr> {
    pub statements: &'expr [Statement<'expr>],
    pub result: Atom,
}

#[derive(Copy, Clone, Debug)]
pub struct Statement<'expr> {
    pub variable: Tag,
    pub r#type: Type<'expr>,
    pub value: Expr<'expr>,
}

#[derive(Copy, Clone, Debug)]
pub enum Expr<'expr> {
    Atom(Atom),
    FieldAccess {
        r#struct: Atom,
        field: Tag,
    },
    Struct {
        fields: &'expr [Field],
    },
    Call {
        function: &'expr Atom,
        arguments: &'expr [Atom],
    },
}

#[derive(Copy, Clone, Debug)]
pub struct Field {
    pub name: Tag,
    pub value: Atom,
}

#[derive(Copy, Clone, Debug)]
pub enum Atom {
    Variable(Tag),
    Literal(Literal),
}

#[derive(Copy, Clone, Debug)]
pub enum Literal {
    Integer(i64),
}

#[derive(Copy, Clone, Debug)]
pub struct StructDefinition<'expr> {
    pub name: Tag,
    pub fields: &'expr [FieldDefinition<'expr>],
}

#[derive(Copy, Clone, Debug)]
pub struct FieldDefinition<'expr> {
    pub name: Tag,
    pub r#type: Type<'expr>,
}

#[derive(Copy, Clone, Debug)]
pub enum Type<'expr> {
    Function {
        arguments: &'expr [Type<'expr>],
        result: &'expr Type<'expr>,
    },
    SharedPointer {
        value: &'expr Type<'expr>,
    },
    Named {
        name: Tag,
    },
    Any {
        metadata: Expr<'expr>,
    },
}
