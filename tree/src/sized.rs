use core::fmt;
use std::ops::{Add, AddAssign};

use crate::generic::{self, DisplayStage, Stage};
use crate::String;

pub use generic::{Field, Generic, Literal, Struct, Type};

#[derive(Clone)]
pub struct Sized;

#[derive(Clone)]
pub struct Call {
    pub result: Type,
    pub size: Size,
    pub witness: Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct Size {
    pub static_size: usize,
    pub dynamic: Vec<Expr>,
}

#[derive(Clone)]
pub struct Argument {
    pub name: String,
    pub typ: Type,
    pub size: Size,
    pub witness: Expr,
}

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub size: Size,
    pub witness: Box<Expr>,
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

impl Size {
    pub fn new_static(static_size: usize) -> Self {
        Self {
            static_size,
            dynamic: Vec::new(),
        }
    }
}

impl Add<Size> for Size {
    type Output = Size;

    fn add(mut self, rhs: Size) -> Self::Output {
        self.dynamic.extend_from_slice(&rhs.dynamic);
        Size {
            static_size: self.static_size + rhs.static_size,
            dynamic: self.dynamic,
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Expr::CallDirect { function, .. } = self.witness.as_ref() {
            if function == "Type" {
                return write!(f, "{}", self.name);
            }
        }
        write!(f, "{} |size {} |wit {}", self.name, self.size, self.witness)
    }
}

impl fmt::Debug for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Expr::CallDirect { function, .. } = &self.witness {
            if function == "Type" {
                return write!(f, "{}: Type", self.name);
            }
        }
        write!(
            f,
            "{}: {} |size {} |wit {}",
            self.name, self.typ, self.size, self.witness
        )
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.static_size != 0 {
            write!(f, "{}", self.static_size)?;
        }
        let mut first = true;
        for d in &self.dynamic {
            if !(first && self.static_size == 0) {
                write!(f, " + ")?;
            }
            write!(f, "{}", d)?;
            first = false;
        }
        Ok(())
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} |size {}", self.result, self.size)?;
        if let Some(wit) = self.witness.as_ref() {
            write!(f, " |wit {}", wit.as_ref())?;
        }
        Ok(())
    }
}
