pub use crate::parsed::Argument;
use crate::String;
use crate::{
    generic::{self, Stage},
    sized::Primitive,
};

pub use generic::{Field, Generic, Literal, Struct, Type};

pub struct Typed;

#[derive(Clone)]
pub struct Call {
    pub result: Type,
    pub generics: Vec<Type>,
}

impl Stage for Typed {
    type Variable = String;
    type Argument = Argument;
    type Call = Call;
    type Type = Type;
}

pub type Program = generic::Program<Typed>;
pub type Function = generic::Function<Typed>;
pub type Expr = generic::Expr<Typed>;
pub type Block = generic::Block<Typed>;
pub type Statement = generic::Statement<Typed>;

impl Expr {
    pub fn get_type(&self) -> Type {
        match self {
            generic::Expr::Variable { typ, .. } => typ.clone(),
            generic::Expr::Literal { literal } => literal.get_type(),
            generic::Expr::CallDirect { tag, .. } => tag.result.clone(),
            generic::Expr::Block(block) => block.result.get_type(),

            generic::Expr::Primitive {
                primitive,
                arguments,
            } =>
            {
                #[allow(irrefutable_let_patterns)]
                if let Primitive::Add | Primitive::Sub | Primitive::Mul = primitive {
                    arguments[0].get_type()
                } else {
                    unimplemented!()
                }
            }
        }
    }
}
