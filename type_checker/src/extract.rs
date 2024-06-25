use ir::qualified::Primitives;
use ir::typed::{Expr, Literal, Type};

pub trait Typeable<'expr> {
    #[must_use]
    fn extract(&self, primitives: &Primitives) -> Type<'expr>;
}

impl<'expr> Typeable<'expr> for Literal {
    fn extract(&self, primitives: &Primitives) -> Type<'expr> {
        match self {
            Literal::Integer(_) => Type::Named {
                name: primitives.integer.clone(),
                arguments: &[],
            },
            Literal::Boolean(_) => Type::Named {
                name: primitives.boolean.clone(),
                arguments: &[],
            },
        }
    }
}

impl<'expr> Typeable<'expr> for Expr<'expr> {
    fn extract(&self, primitives: &Primitives) -> Type<'expr> {
        match self {
            Expr::Variable { r#type, .. } => r#type.clone(),
            Expr::Literal { literal, .. } => literal.extract(primitives),
            Expr::Call { r#type, .. } => r#type.clone(),
            Expr::Operation { .. } => todo!(),
            Expr::StructLiteral { name, .. } => Type::Named {
                name: name.clone(),
                arguments: todo!(),
            },
            Expr::Block(block) => block.result.expect("todo").extract(primitives),
            Expr::Annotated { annotation, .. } => annotation.clone(),
            Expr::Case { branches, .. } => branches.iter().next().unwrap().body.extract(primitives),
        }
    }
}
