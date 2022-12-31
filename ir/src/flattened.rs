use crate::typed::Identifier;

#[derive(Copy, Clone, Debug)]
pub struct Program<'expr, 'ident> {
    pub blocks: &'expr [Block<'expr, 'ident>],
    pub definitions: &'expr [Definition<'expr, 'ident>],
}

#[derive(Copy, Clone, Debug)]
pub struct Definition<'expr, 'ident> {
    pub blocks: &'expr [Block<'expr, 'ident>],
    pub name: &'ident str,
}

#[derive(Copy, Clone, Debug)]
pub struct Block<'expr, 'ident> {
    pub tag: Label,
    pub statements: &'expr [Statement<'expr, 'ident>],
    pub terminator: Terminator<'expr, 'ident>,
}

#[derive(Copy, Clone, Debug)]
pub enum Terminator<'expr, 'ident> {
    Call(Call<'expr, 'ident>),
    Terminate {
        result: Atom<'expr, 'ident>,
    },
    Switch {
        predicate: Atom<'expr, 'ident>,
        branches: &'expr [Branch<'expr, 'ident>],
    },
}

#[derive(Copy, Clone, Debug)]
pub struct Branch<'expr, 'ident> {
    pub pattern: Pattern,
    pub result: Call<'expr, 'ident>,
}

#[derive(Copy, Clone, Debug)]
pub struct Call<'expr, 'ident> {
    pub function: CallTarget<'expr, 'ident>,
    pub arguments: &'expr [Atom<'expr, 'ident>],
}

#[derive(Copy, Clone, Debug)]
pub enum Pattern {
    Default,
}

#[derive(Copy, Clone, Debug)]
pub enum CallTarget<'expr, 'ident> {
    Direct(Label),
    Indirect(Identifier<'expr, 'ident>),
}

#[derive(Copy, Clone, Debug)]
pub struct Label {
    pub tag: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Statement<'expr, 'ident> {
    pub variable: Identifier<'expr, 'ident>,
    pub value: Expr<'expr, 'ident>,
}

#[derive(Copy, Clone, Debug)]
pub enum Expr<'expr, 'ident> {
    Atom(Atom<'expr, 'ident>),
    StructLiteral(&'expr [Field<'expr, 'ident>]),
    FieldAccess {
        r#struct: Atom<'expr, 'ident>,
        field: &'ident str,
    },
}

#[derive(Copy, Clone, Debug)]
pub struct Field<'expr, 'ident> {
    pub name: &'ident str,
    pub value: Atom<'expr, 'ident>,
}

#[derive(Copy, Clone, Debug)]
pub enum Atom<'expr, 'ident> {
    Variable(Identifier<'expr, 'ident>),
    Literal(Literal),
}

#[derive(Copy, Clone, Debug)]
pub enum Literal {
    Integer(i64),
}
