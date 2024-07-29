use core::fmt;

use crate::String;

pub trait Stage {
    type Argument: Clone;
    type Call: Clone;
    type Type: Clone;
    type Variable: Clone;
}

pub trait DisplayStage:
    Stage<
    Argument = <Self as DisplayStage>::Argument,
    Call = <Self as DisplayStage>::Call,
    Type = <Self as DisplayStage>::Type,
    Variable = <Self as DisplayStage>::Variable,
>
{
    type Argument: Clone + fmt::Debug;
    type Call: Clone + fmt::Display;
    type Type: Clone + fmt::Display;
    type Variable: Clone + fmt::Display;
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

#[derive(Clone, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug)]
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
pub enum Primitive {
    Add,
    Sub,
}

impl Primitive {
    pub fn arity(&self) -> Option<usize> {
        use Primitive::*;
        match self {
            Add | Sub => Some(2),
        }
    }
}

#[derive(Clone)]
pub enum Literal {
    Float(f64),
    Integer(i64),
}
impl Literal {
    pub fn get_type(&self) -> Type {
        match self {
            Literal::Float(_) => Type::float(),
            Literal::Integer(_) => Type::integer(),
        }
    }
}

#[derive(Clone)]
pub enum Expr<S: Stage> {
    Variable {
        name: S::Variable,
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
        name: S::Variable,
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

impl Type {
    pub fn typ() -> Self {
        Self::Named {
            name: String::from("Type"),
            arguments: Vec::new(),
        }
    }
    pub fn float() -> Self {
        Self::Named {
            name: String::from("F64"),
            arguments: Vec::new(),
        }
    }
    pub fn bool() -> Self {
        Self::Named {
            name: String::from("Bool"),
            arguments: Vec::new(),
        }
    }

    pub fn integer() -> Type {
        Self::Named {
            name: String::from("I64"),
            arguments: Vec::new(),
        }
    }
}

impl<S: DisplayStage> fmt::Display for Program<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.structs {
            write!(f, "{}", s)?;
        }
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }
        Ok(())
    }
}

impl<S: DisplayStage> fmt::Display for Function<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func {}", self.name)?;
        f.debug_list().entries(self.generics.iter()).finish()?;
        write!(f, "(")?;
        for (i, arg) in self.arguments.iter().enumerate() {
            if i != 0 {
                write!(f, ",\n\t")?;
            } else {
                write!(f, "\n\t")?;
            }
            write!(f, "{:?}", arg)?;
        }
        write!(f, "\n): {:?} = {}", self.result, self.body)
    }
}

impl<S: DisplayStage> fmt::Display for Expr<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Variable { name, typ } => write!(f, "({}: {})", name, typ),
            Expr::Literal { literal } => write!(f, "{}", literal),
            Expr::CallDirect {
                function,
                arguments,
                tag,
            } => {
                write!(f, "{}[{}](", function, tag)?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Block(block) => block.fmt(f),
        }
    }
}

impl<S: DisplayStage> fmt::Display for Block<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for stmt in &self.stmts {
            write!(f, "\n\t{stmt}")?;
        }
        write!(f, "\n}}")
    }
}

impl<S: DisplayStage> fmt::Display for Statement<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let { name, typ, value } => {
                write!(f, "let {name}: {typ} = {value}")
            }
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Float(float) => write!(f, "{}", float),
            Literal::Integer(integer) => write!(f, "{}", integer),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Primitive::Add => "+",
            Primitive::Sub => "-",
        }
        .fmt(f)
    }
}

impl<S: Stage> fmt::Debug for Expr<S>
where
    Expr<S>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct(&format!("struct {}", self.name));
        for field in &self.fields {
            s.field(&field.name, &field.typ);
        }
        s.finish()
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Named { name, arguments } => {
                write!(f, "{}", name)?;
                if !arguments.is_empty() {
                    f.debug_list().entries(arguments.iter()).finish()
                } else {
                    Ok(())
                }
            }
            Type::Generic { name } => write!(f, "{}", name),
            Type::Function { arguments, result } => {
                if arguments.len() == 1 {
                    write!(f, "{:?}", arguments[0])?;
                } else {
                    let mut tuple = f.debug_tuple("");
                    for arg in arguments {
                        tuple.field(arg);
                    }
                    tuple.finish()?;
                }
                write!(f, " -> {:?}", result.as_ref())
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
