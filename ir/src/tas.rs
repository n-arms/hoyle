use tree::typed::Literal;
use tree::String;

#[derive(Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Clone)]
pub struct Function {
    pub arguments: Vec<Variable>,
}

#[derive(Clone)]
pub struct Argument {
    pub variable: Variable,
    pub convention: Convention,
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
pub struct CallArgument {
    value: Atom,
    convention: Convention,
}

#[derive(Copy, Clone)]
pub enum Convention {
    In,
    Inout,
}

#[derive(Clone)]
pub enum Instr {
    CallDirect {
        name: String,
        arguments: Vec<CallArgument>,
    },
    CallIndirect {
        function: Atom,
        arguments: Vec<CallArgument>,
    },
    Set {
        target: Target,
    },
}

#[derive(Clone)]
pub enum Target {
    Variable(Variable),
    Deref(Variable),
}

#[derive(Clone)]
pub enum Atom {
    Variable(Variable),
    Literal(Literal),
    Function(String),
}

#[derive(Clone)]
pub enum Type {
    Trivial(usize),
    Tuple(Vec<Type>),
    Pointer(Box<Type>),
    Function(Vec<Type>),
}
