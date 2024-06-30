use core::fmt;
use std::ops::{Add, AddAssign};

use tree::sized::{Literal, Struct, Type};
use tree::String;

#[derive(Clone)]
pub struct Program {
    pub structs: Vec<Struct>,
    pub functions: Vec<Function>,
}

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub arguments: Vec<Variable>,
    pub body: Block,
}

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub typ: Type,
    pub size: Size,
    pub offset: Size,
    pub witness: Option<Box<Variable>>,
}

#[derive(Clone)]
pub struct Size {
    pub static_size: usize,
    pub dynamic: Vec<Variable>,
}

#[derive(Clone)]
pub struct Block {
    pub instrs: Vec<Instr>,
}

#[derive(Clone)]
pub enum Instr {
    CallDirect { function: String },
    Copy { target: Variable, value: Variable },
    Destory { value: Variable },
    Set { target: Variable, expr: Expr },
}

#[derive(Clone)]
pub enum Expr {
    Literal(Literal),
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

    fn add(self, rhs: Size) -> Self::Output {
        let mut dynamic = self.dynamic;
        dynamic.extend(rhs.dynamic);
        Self {
            static_size: self.static_size + rhs.static_size,
            dynamic,
        }
    }
}

impl AddAssign<Size> for Size {
    fn add_assign(&mut self, rhs: Size) {
        *self = self.clone() + rhs;
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.structs {
            s.fmt(f)?;
        }
        for func in &self.functions {
            func.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func {}(", self.name)?;
        for (i, arg) in self.arguments.iter().enumerate() {
            if i != 0 {
                write!(f, ",\n\t")?;
            } else {
                write!(f, "\n\t")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, "\n) = {{\n{}}}", self.body)
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.typ == Type::typ() {
            write!(f, "{}: {:?} |off {}", self.name, self.typ, self.offset)
        } else {
            write!(
                f,
                "{}: {:?} |size {} |off {}",
                self.name, self.typ, self.size, self.offset
            )?;
            if let Some(witness) = self.witness.as_ref() {
                write!(f, " |wit ({})", witness.as_ref())
            } else {
                Ok(())
            }
        }
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
            write!(f, "({})", d)?;
            first = false;
        }
        Ok(())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for instr in &self.instrs {
            write!(f, "\t{}\n", instr)?;
        }
        Ok(())
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instr::CallDirect { function } => write!(f, "call {}", function),
            Instr::Copy { target, value } => write!(f, "{} <- copy {}", target, value),
            Instr::Destory { value } => write!(f, "destroy {}", value),
            Instr::Set { target, expr } => write!(f, "{} = {}", target, expr),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(literal) => match literal {
                Literal::Float(float) => float.fmt(f),
            },
        }
    }
}
