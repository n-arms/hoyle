use core::fmt;

use tree::sized::{Literal, Primitive, Struct, Type};
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
    Type,
}

#[derive(Clone)]
pub struct Instr {
    pub target: Variable,
    pub value: Expr,
}

impl Instr {
    pub fn new(target: Variable, value: Expr) -> Self {
        Self { target, value }
    }
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
    Primitive(Primitive, Vec<Variable>),
    CallDirect {
        function: String,
        arguments: Vec<CallArgument>,
    },
    Move {
        source: Variable,
        witness: Witness,
    },
    Copy {
        source: Variable,
        witness: Witness,
    },
    Destroy {
        witness: Witness,
    },
}

#[derive(Clone)]
pub struct CallArgument {
    pub value: Variable,
    pub convention: Convention,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Convention {
    In,
    Inout,
    Out,
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
        write!(f, "\n) = body {{\n {}}}", self.body)
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
            Witness::Type => write!(f, "Type"),
        }
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.target, self.value)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Primitive(prim, terms) => match prim.arity() {
                Some(2) => {
                    assert_eq!(terms.len(), 2);
                    write!(f, "{} {} {}", terms[0], prim, terms[1])
                }
                _ => {
                    write!(f, "{}", prim)?;
                    for term in terms {
                        write!(f, " {}", term)?;
                    }
                    Ok(())
                }
            },
            Expr::CallDirect {
                function,
                arguments,
            } => {
                let mut tuple = f.debug_tuple(function.as_str());
                for arg in arguments {
                    tuple.field(arg);
                }
                tuple.finish()
            }
            Expr::Move { source, witness } => write!(f, "move {source} using {witness}"),
            Expr::Copy { source, witness } => write!(f, "copy {source} using {witness}"),
            Expr::Destroy { witness } => write!(f, "destroy using {witness}"),
            Expr::Literal(literal) => write!(f, "{}", literal),
        }
    }
}

impl fmt::Debug for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for CallArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.convention, self.value)
    }
}

impl fmt::Display for Convention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Convention::In => write!(f, "in"),
            Convention::Inout => write!(f, "inout"),
            Convention::Out => write!(f, "out"),
        }
    }
}

impl fmt::Debug for CallArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
