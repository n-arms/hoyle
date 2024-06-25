use crate::generic::{self, Stage};
use crate::String;

pub use generic::{Field, Generic, Literal, Struct, Type};

#[derive(Clone)]
pub struct Parsed;

#[derive(Clone)]
pub struct Argument {
    pub name: String,
    pub typ: Type,
}

impl Stage for Parsed {
    type Argument = Argument;
    type Call = ();
    type Type = ();
}

pub type Program = generic::Program<Parsed>;
pub type Function = generic::Function<Parsed>;
pub type Expr = generic::Expr<Parsed>;
pub type Statement = generic::Statement<Parsed>;
pub type Block = generic::Block<Parsed>;
