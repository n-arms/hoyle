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
}

#[derive(Clone)]
pub struct Block {
    pub instrs: Vec<Instr>,
}

#[derive(Clone)]
pub enum Witness {
    Trivial { size: usize },
    Dynamic { location: Variable },
}

#[derive(Clone)]
pub enum Instr {
    CallDirect {
        function: String,
        arguments: Vec<Variable>,
    },
    Copy {
        target: Variable,
        value: Variable,
        witness: Witness,
    },
    Destory {
        value: Variable,
        witness: Witness,
    },
    Set {
        target: Variable,
        expr: Expr,
    },
}

impl Witness {
    pub fn trivial(size: usize) -> Self {
        Self::Trivial { size }
    }

    pub fn is_trivial(&self) -> bool {
        matches!(self, Self::Trivial { .. })
    }
}

#[derive(Clone)]
pub enum Expr {
    Literal(Literal),
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
            write!(f, "{}: {:?}", self.name, self.typ)
        } else {
            write!(f, "{}: {:?}", self.name, self.typ)
        }
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

impl fmt::Display for Witness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Witness::Trivial { size } => write!(f, "trivial {}", size),
            Witness::Dynamic { location } => location.name.fmt(f),
        }
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instr::CallDirect {
                function,
                arguments,
            } => {
                write!(f, "call {} with ", function)?;
                let mut first = true;
                for arg in arguments {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                    first = false;
                }
                Ok(())
            }
            Instr::Copy {
                target,
                value,
                witness,
            } => {
                write!(f, "{} <- copy {} using {}", target, value, witness)
            }
            Instr::Destory { value, witness } => {
                write!(f, "destroy {} using {}", value, witness)
            }
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
