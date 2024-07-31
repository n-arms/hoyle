use core::fmt;

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

#[derive(Copy, Clone)]
pub struct If;

impl Stage for Parsed {
    type Variable = String;
    type Argument = Argument;
    type Call = ();
    type Type = ();
    type StructPack = ();
    type If = If;
}

pub type Program = generic::Program<Parsed>;
pub type Function = generic::Function<Parsed>;
pub type Expr = generic::Expr<Parsed>;
pub type Statement = generic::Statement<Parsed>;
pub type Block = generic::Block<Parsed>;
pub type PackField = generic::PackField<Parsed>;

impl fmt::Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
