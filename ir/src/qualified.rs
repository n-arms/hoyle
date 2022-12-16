use crate::ast;

#[derive(Copy, Clone, Debug)]
pub struct Identifier<'expr, 'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
    pub r#type: Type<'expr, 'ident>,
    pub span: ast::Span,
}

#[derive(Copy, Clone, Debug)]
pub enum Path {
    Current,
}

#[derive(Copy, Clone, Debug)]
pub enum IdentifierSource {
    Local,
    Global(Path),
}

#[derive(Copy, Clone, Debug)]
pub enum Type<'expr, 'ident> {
    Named {
        source: IdentifierSource,
        name: &'ident str,
        span: ast::Span,
    },
    Tuple(&'expr [Type<'expr, 'ident>], ast::Span),
}

pub type Program<'expr, 'ident> =
    ast::Program<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, Identifier<'expr, 'ident>>;

pub type Block<'expr, 'ident> = ast::Block<'expr, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Expr<'expr, 'ident> = ast::Expr<'expr, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;
