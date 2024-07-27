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
    pub result: Atom,
}

#[derive(Clone)]
pub enum Witness {
    Trivial { size: usize },
    Dynamic { location: Variable },
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
pub enum Atom {
    Literal(Literal),
    Variable(Variable),
}

#[derive(Clone)]
pub enum Expr {
    Atom(Atom),
    Primitive(Primitive, Vec<Atom>),
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
    Destory {
        witness: Witness,
    },
}

#[derive(Clone)]
pub struct CallArgument {
    pub value: Atom,
    pub convention: Convention,
}

#[derive(Copy, Clone)]
pub enum Convention {
    In,
    Inout,
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
        write!(f, "\t{}\n", self.result)
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
            Expr::Atom(atom) => write!(f, "{atom}"),
            Expr::CallDirect {
                function,
                arguments,
            } => {
                let mut tuple = f.debug_tuple(function.as_str());
                for arg in arguments {
                    tuple.field(arg);
                }
                Ok(())
            }
            Expr::Move { source, witness } => write!(f, "move {source} using {witness}"),
            Expr::Copy { source, witness } => write!(f, "copy {source} using {witness}"),
            Expr::Destory { witness } => write!(f, "destory using {witness}"),
        }
    }
}

impl fmt::Debug for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Literal(literal) => write!(f, "{literal}"),
            Atom::Variable(variable) => write!(f, "{variable}"),
        }
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
        }
    }
}

impl fmt::Debug for CallArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
