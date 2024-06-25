use crate::String;

pub trait Stage {
    type Argument: Clone;
    type Call: Clone;
    type Type: Clone;
}

#[derive(Clone)]
pub struct Program<S: Stage> {
    pub structs: Vec<Struct>,
    pub functions: Vec<Function<S>>,
}

#[derive(Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub typ: Type,
}

#[derive(Clone)]
pub enum Type {
    Named {
        name: String,
        arguments: Vec<Type>,
    },
    Generic {
        name: String,
    },
    Function {
        arguments: Vec<Type>,
        result: Box<Type>,
    },
}

#[derive(Clone)]
pub struct Generic {
    pub name: String,
}

#[derive(Clone)]
pub struct Function<S: Stage> {
    pub name: String,
    pub generics: Vec<Generic>,
    pub arguments: Vec<S::Argument>,
    pub result: Type,
    pub body: Expr<S>,
}

#[derive(Clone)]
pub enum Literal {
    Float(f64),
}

#[derive(Clone)]
pub enum Expr<S: Stage> {
    Variable {
        name: String,
        typ: S::Type,
    },
    Literal {
        literal: Literal,
    },
    CallDirect {
        function: String,
        arguments: Vec<Expr<S>>,
        tag: S::Call,
    },
    Block(Block<S>),
}

#[derive(Clone)]
pub enum Statement<S: Stage> {
    Let {
        name: String,
        typ: S::Type,
        value: Expr<S>,
    },
}

#[derive(Clone)]
pub struct Block<S: Stage> {
    pub stmts: Vec<Statement<S>>,
    pub result: Box<Expr<S>>,
}

impl<S: Stage> Program<S> {
    pub fn from_struct(struct_def: Struct) -> Self {
        Self {
            structs: vec![struct_def],
            functions: Vec::new(),
        }
    }

    pub fn from_function(function_def: Function<S>) -> Self {
        Self {
            structs: Vec::new(),
            functions: vec![function_def],
        }
    }
}
