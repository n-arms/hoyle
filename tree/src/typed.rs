use crate::generic::{self, Stage};
use crate::parsed::Argument;
use crate::String;

pub use generic::{Block, Field, Generic, Literal, Statement, Struct, Type};

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
