use core::fmt;

pub use crate::parsed::Argument;
use crate::parsed::If;
use crate::String;
use crate::{
    generic::{self, Stage},
    sized::Primitive,
};

pub use generic::{Field, Generic, Literal, Type};

#[derive(Clone)]
pub struct Typed;

#[derive(Clone)]
pub struct Call {
    pub result: Type,
    pub generics: Vec<Type>,
}

#[derive(Clone)]
pub struct StructPack {
    pub result: Type,
    pub generics: Vec<Type>,
}

#[derive(Clone)]
pub struct Closure {
    pub captures: Vec<ClosureArgument>,
    pub result: Type,
}

#[derive(Clone)]
pub struct ClosureArgument {
    pub name: String,
    pub typ: Type,
}

impl Stage for Typed {
    type Variable = String;
    type Argument = Argument;
    type Call = Call;
    type Type = Type;
    type StructPack = StructPack;
    type If = If;
    type StructMeta = ();
    type Closure = Closure;
    type ClosureArgument = ClosureArgument;
}

pub type Program = generic::Program<Typed>;
pub type Function = generic::Function<Typed>;
pub type Expr = generic::Expr<Typed>;
pub type Block = generic::Block<Typed>;
pub type Statement = generic::Statement<Typed>;
pub type PackField = generic::PackField<Typed>;
pub type Struct = generic::Struct<Typed>;

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
            generic::Expr::Closure { tag, .. } => tag.result.clone(),
        }
    }
}

impl fmt::Display for ClosureArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.typ)
    }
}
