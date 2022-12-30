use crate::ast;
use crate::qualified::{self, IdentifierSource, Path, TypeName};
use arena_alloc::{General, Interning, Specialized};
use std::fmt::{Debug, Formatter};

pub type Type<'expr, 'ident> = qualified::Type<'expr, 'ident, Option<ast::Span>>;

#[derive(Copy, Clone)]
pub struct Identifier<'expr, 'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
    pub r#type: Type<'expr, 'ident>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct UntypedIdentifier<'ident> {
    pub source: IdentifierSource,
    pub name: &'ident str,
}

pub type Program<'expr, 'ident> =
    ast::Program<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Definition<'expr, 'ident> =
    ast::Definition<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Argument<'expr, 'ident> =
    ast::Argument<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Statement<'expr, 'ident> =
    ast::Statement<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Pattern<'expr, 'ident> = ast::Pattern<'expr, 'ident, Identifier<'expr, 'ident>>;

pub type FieldDefinition<'expr, 'ident> = ast::FieldDefinition<'ident, Type<'expr, 'ident>>;

pub type Block<'expr, 'ident> =
    ast::Block<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Field<'expr, 'ident> =
    ast::Field<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Branch<'expr, 'ident> =
    ast::Branch<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type Expr<'expr, 'ident> =
    ast::Expr<'expr, 'ident, Identifier<'expr, 'ident>, Type<'expr, 'ident>>;

pub type PatternField<'expr, 'ident> = ast::PatternField<'expr, 'ident, Identifier<'expr, 'ident>>;

impl Debug for Identifier<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)?;
        write!(f, "::")?;
        self.name.fmt(f)?;
        write!(f, " : ")?;
        self.r#type.fmt(f)
    }
}

impl<'ident> From<qualified::Identifier<'_, 'ident>> for UntypedIdentifier<'ident> {
    fn from(id: qualified::Identifier<'_, 'ident>) -> Self {
        Self {
            source: id.source,
            name: id.name,
        }
    }
}

impl<'expr, 'ident> Identifier<'expr, 'ident> {
    pub fn new(id: impl Into<UntypedIdentifier<'ident>>, id_type: Type<'expr, 'ident>) -> Self {
        let untyped_id = id.into();
        Self {
            source: untyped_id.source,
            name: untyped_id.name,
            r#type: id_type,
        }
    }
}

impl<'expr, 'ident> ast::Literal<'expr> {
    #[must_use] pub fn r#type(self, interner: &Interning<'ident, Specialized>) -> Type<'expr, 'ident> {
        Type::Named {
            name: TypeName {
                name: interner.get_or_intern("int"),
                source: IdentifierSource::Global(Path::Builtin),
            },
            span: None,
        }
    }
}

impl<'expr, 'ident> Expr<'expr, 'ident> {
    #[must_use] pub fn r#type(
        self,
        interner: &Interning<'ident, Specialized>,
        _general: &General<'expr>,
    ) -> Type<'expr, 'ident> {
        match self {
            ast::Expr::Variable(id, _) => id.r#type,
            ast::Expr::Literal(literal, _) => literal.r#type(interner),
            ast::Expr::Call {
                function: _,
                arguments: _,
                span: _,
            } => todo!(),
            ast::Expr::Operation {
                operator: _,
                arguments: _,
                span: _,
            } => todo!(),
            ast::Expr::StructLiteral { name: _, fields: _, span: _ } => todo!(),
            ast::Expr::Block(_) => todo!(),
            ast::Expr::Annotated {
                expr: _,
                annotation: _,
                span: _,
            } => todo!(),
            ast::Expr::Case {
                predicate: _,
                branches: _,
                span: _,
            } => todo!(),
        }
    }
}
