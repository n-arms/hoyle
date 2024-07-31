pub use crate::parsed::Argument;
use crate::parsed::If;
use crate::typed::StructPack;
use crate::String;
use crate::{
    generic::{self, Stage},
    sized::Primitive,
};

pub use generic::{Field, Generic, Literal, Struct, Type};

#[derive(Copy, Clone)]
pub struct TypePassing;

#[derive(Clone)]
pub struct Call {
    pub result: Type,
}

impl Stage for TypePassing {
    type Variable = String;
    type Argument = Argument;
    type Call = Call;
    type Type = Type;
    type StructPack = StructPack;
    type If = If;
}

pub type Program = generic::Program<TypePassing>;
pub type Function = generic::Function<TypePassing>;
pub type Expr = generic::Expr<TypePassing>;
pub type Block = generic::Block<TypePassing>;
pub type Statement = generic::Statement<TypePassing>;
pub type StructBuilder = generic::StructBuilder<TypePassing>;
pub type StructBuilders = generic::StructBuilders<TypePassing>;
pub type PackField = generic::PackField<TypePassing>;

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
            generic::Expr::StructPack { tag, .. } => tag.result.clone(),
            generic::Expr::If { true_branch, .. } => true_branch.get_type(),
        }
    }
}
