use crate::env::Primitives;
use ir::ast::Literal;
use ir::qualified::{self, Type};
use ir::typed::Expr;

pub trait Typeable<'expr, 'ident> {
    #[must_use]
    fn extract(&self, primitives: &Primitives<'expr, 'ident>) -> Type<'expr, 'ident>;
}

impl<'expr, 'ident> Typeable<'expr, 'ident> for Literal<'expr> {
    fn extract(&self, primitives: &Primitives<'expr, 'ident>) -> Type<'expr, 'ident> {
        match self {
            Literal::Integer(_) => primitives.int,
        }
    }
}

impl<'expr, 'ident> Typeable<'expr, 'ident> for Expr<'expr, 'ident> {
    fn extract(&self, primitives: &Primitives<'expr, 'ident>) -> Type<'expr, 'ident> {
        match self {
            ir::ast::Expr::Variable(id, _) => id.r#type,
            ir::ast::Expr::Literal(literal, _) => literal.extract(primitives),
            ir::ast::Expr::Call { function, .. } => match function.extract(primitives) {
                Type::Arrow { return_type, .. } => *return_type,
                Type::Named { .. } => {
                    panic!("illegal function call: {function:?} is not an arrow type")
                }
            },
            ir::ast::Expr::Operation { .. } => todo!(),
            ir::ast::Expr::StructLiteral { name, .. } => struct_type(name.identifier),
            ir::ast::Expr::Block(block) => block.result.expect("todo").extract(primitives),
            ir::ast::Expr::Annotated { annotation, .. } => *annotation,
            ir::ast::Expr::Case { branches, .. } => {
                branches.iter().next().unwrap().body.extract(primitives)
            }
        }
    }
}

#[must_use]
pub const fn struct_type<'new, 'ident>(name: qualified::Identifier<'ident>) -> Type<'new, 'ident> {
    Type::Named { name, span: None }
}
