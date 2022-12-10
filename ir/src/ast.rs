#[derive(Copy, Clone, Debug)]
pub struct Program<'ident, 'expr> {
    pub definitions: &'expr [Definition<'ident, 'expr>],
}

#[derive(Copy, Clone, Debug)]
pub struct Definition<'ident, 'expr> {
    pub name: &'ident str,
    pub generics: &'expr [Generic<'ident>],
    pub arguments: &'expr [Argument<'ident, 'expr>],
    pub body: &'expr [Statement<'ident, 'expr>],
}

#[derive(Copy, Clone, Debug)]
pub struct Argument<'ident, 'expr> {
    pub identifier: &'ident str,
    pub type_annotation: Type<'ident, 'expr>,
}

#[derive(Copy, Clone, Debug)]
pub enum Type<'ident, 'expr> {
    Constructor(&'ident str),
    Tuple(&'expr [Type<'ident, 'expr>]),
}

#[derive(Copy, Clone, Debug)]
pub struct Generic<'ident> {
    pub identifier: &'ident str,
}

#[derive(Copy, Clone, Debug)]
pub enum Statement<'ident, 'expr> {
    Let {
        left_side: Pattern<'ident, 'expr>,
        right_side: Expr<'ident, 'expr>,
    },
    Raw(Expr<'ident, 'expr>),
}

#[derive(Copy, Clone, Debug)]
pub enum Pattern<'ident, 'expr> {
    Variable(&'ident str),
    Tuple(&'expr [Pattern<'ident, 'expr>]),
}

#[derive(Copy, Clone, Debug)]
pub enum Expr<'ident, 'expr> {
    Variable(&'ident str),
    Literal(Literal<'ident>),
    Call {
        function: &'expr Expr<'ident, 'expr>,
        arguments: &'expr [Expr<'ident, 'expr>],
    },
    Operation {
        operator: Operator,
        arguments: &'expr [Expr<'ident, 'expr>],
    },
    Block(&'expr [Statement<'ident, 'expr>]),
}

#[derive(Copy, Clone, Debug)]
pub enum Literal<'ident> {
    Integer(&'ident str),
}

#[derive(Copy, Clone, Debug)]
pub enum Operator {
    Add,
    Sub,
    Times,
    Div,
}
