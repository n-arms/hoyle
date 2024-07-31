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
    Type,
}

#[derive(Clone)]
pub struct Call {
    pub result: Type,
    pub witness: Witness,
}

#[derive(Clone)]
pub struct StructPack {
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

#[derive(Clone)]
pub struct If {
    pub witness: Witness,
}

impl Stage for Sized {
    type Variable = Variable;
    type Argument = Argument;
    type Call = Call;
    type Type = Type;
    type StructPack = StructPack;
    type If = If;
}

impl DisplayStage for Sized {
    type Argument = Argument;
    type Call = Call;
    type Type = Type;
    type Variable = Variable;
    type StructPack = StructPack;
    type If = If;
}

pub type Program = generic::Program<Sized>;
pub type Function = generic::Function<Sized>;
pub type Expr = generic::Expr<Sized>;
pub type Block = generic::Block<Sized>;
pub type Statement = generic::Statement<Sized>;
pub type StructBuilder = generic::StructBuilder<Sized>;
pub type StructBuilders = generic::StructBuilders<Sized>;
pub type PackField = generic::PackField<Sized>;

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

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.witness)
    }
}

impl fmt::Display for Witness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Witness::Trivial { size } => write!(f, "{}", size),
            Witness::Dynamic { value } => write!(f, "[{}]", value.as_ref()),
            Witness::Type => write!(f, "Type"),
        }
    }
}

impl fmt::Debug for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}: {}", self.name, self.witness, self.typ)
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{}", self.result, self.witness)
    }
}

impl fmt::Display for StructPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{}", self.result, self.witness)
    }
}

impl fmt::Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.witness)
    }
}
