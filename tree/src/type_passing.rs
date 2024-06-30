use crate::generic::{self, Stage};
pub use crate::parsed::Argument;
use crate::String;

pub use generic::{Field, Generic, Literal, Struct, Type};

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
}

pub type Program = generic::Program<TypePassing>;
pub type Function = generic::Function<TypePassing>;
pub type Expr = generic::Expr<TypePassing>;
pub type Block = generic::Block<TypePassing>;
pub type Statement = generic::Statement<TypePassing>;

impl Expr {
    pub fn get_type(&self) -> Type {
        match self {
            generic::Expr::Variable { typ, .. } => typ.clone(),
            generic::Expr::Literal { literal } => literal.get_type(),
            generic::Expr::CallDirect { tag, .. } => tag.result.clone(),
            generic::Expr::Block(block) => block.result.get_type(),
        }
    }
}
