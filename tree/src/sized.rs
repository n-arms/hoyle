use core::fmt;
use std::ops::{Add, AddAssign};

use crate::generic::{self, DisplayStage, Stage};
use crate::String;

pub use generic::{Field, Generic, Literal, Primitive, Struct, Type};

#[derive(Clone)]
pub struct Sized;

#[derive(Clone)]
pub enum Witness {
    Trivial { size: usize },
    Dynamic { value: Box<Expr> },
}

#[derive(Clone)]
pub struct Call {
    pub result: Type,
    pub witness: Witness,
}

#[derive(Clone)]
pub struct Argument {
    pub name: String,
    pub typ: Type,
    pub witness: Witness,
}

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub witness: Witness,
}

impl Stage for Sized {
    type Variable = Variable;
    type Argument = Argument;
    type Call = Call;
    type Type = Type;
}

impl DisplayStage for Sized {
    type Argument = Argument;
    type Call = Call;
    type Type = Type;
    type Variable = Variable;
}

pub type Program = generic::Program<Sized>;
pub type Function = generic::Function<Sized>;
pub type Expr = generic::Expr<Sized>;
pub type Block = generic::Block<Sized>;
pub type Statement = generic::Statement<Sized>;

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

impl Witness {
    pub fn typ() -> Self {
        Self::Trivial { size: 24 }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "{}@{}", self.name, self.witness)
        write!(f, "{}@", self.name)
    }
}

impl fmt::Display for Witness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Witness::Trivial { size } => write!(f, "{}", size),
            Witness::Dynamic { value } => write!(f, "[{}]", value.as_ref()),
        }
    }
}

impl fmt::Debug for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "{}@{}: {}", self.name, self.witness, self.typ)
        //write!(f, "{}: {}", self.name, self.typ)
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "{:?}@{}", self.result, self.witness)
        write!(f, "{:?}", self.result)
    }
}
