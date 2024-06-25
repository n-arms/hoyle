use crate::generic::{self, Stage};
pub use crate::parsed::Argument;
use crate::String;

pub use generic::{Field, Generic, Literal, Struct, Type};

pub struct Typed;

#[derive(Clone)]
pub struct Call {
    pub result: Type,
    pub generics: Vec<Type>,
}

impl Stage for Typed {
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
            generic::Expr::Literal { .. } => todo!(),
            generic::Expr::CallDirect { tag, .. } => tag.result.clone(),
            generic::Expr::Block(block) => block.result.get_type(),
        }
    }
}
