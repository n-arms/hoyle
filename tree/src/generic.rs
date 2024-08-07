use core::{fmt, hash};
use std::{cell::OnceCell, rc::Rc};

use crate::String;

pub trait Stage {
    type Argument: Clone;
    type Call: Clone;
    type Type: Clone;
    type Variable: Clone;
    type StructPack: Clone;
    type If: Clone;
    type StructMeta: Clone;
    type Closure: Clone;
    type ClosureArgument: Clone;
}

pub trait DisplayStage:
    Stage<
    Argument = <Self as DisplayStage>::Argument,
    Call = <Self as DisplayStage>::Call,
    Type = <Self as DisplayStage>::Type,
    Variable = <Self as DisplayStage>::Variable,
    StructPack = <Self as DisplayStage>::StructPack,
    If = <Self as DisplayStage>::If,
    StructMeta = <Self as DisplayStage>::StructMeta,
    Closure = <Self as DisplayStage>::Closure,
    ClosureArgument = <Self as DisplayStage>::ClosureArgument,
>
{
    type Argument: Clone + fmt::Debug;
    type Call: Clone + fmt::Display;
    type Type: Clone + fmt::Display;
    type Variable: Clone + fmt::Display;
    type StructPack: Clone + fmt::Display;
    type If: Clone + fmt::Display;
    type StructMeta: Clone + fmt::Display;
    type Closure: Clone + fmt::Display;
    type ClosureArgument: Clone + fmt::Debug;
}

#[derive(Clone)]
pub struct Program<S: Stage> {
    pub structs: Vec<Struct<S>>,
    pub functions: Vec<Function<S>>,
}

#[derive(Clone)]
pub struct Struct<S: Stage> {
    pub name: String,
    pub fields: Vec<Field>,
    pub tag: S::StructMeta,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Convention {
    In,
    Inout,
    Out,
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
    Unification {
        name: String,
        value: Rc<OnceCell<Type>>,
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

#[derive(Copy, Clone)]
pub enum Primitive {
    Add,
    Sub,
    Mul,
}

impl Primitive {
    pub fn arity(&self) -> Option<usize> {
        use Primitive::*;
        match self {
            Add | Sub | Mul => Some(2),
        }
    }
}

#[derive(Clone)]
pub enum Literal {
    Float(f64),
    Integer(i64),
    Boolean(bool),
}
impl Literal {
    pub fn get_type(&self) -> Type {
        match self {
            Literal::Float(_) => Type::float(),
            Literal::Integer(_) => Type::integer(),
            Literal::Boolean(_) => Type::bool(),
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
    Primitive {
        primitive: Primitive,
        arguments: Vec<Expr<S>>,
    },
    Block(Block<S>),
    StructPack {
        name: String,
        fields: Vec<PackField<S>>,
        tag: S::StructPack,
    },
    If {
        predicate: Box<Expr<S>>,
        true_branch: Box<Expr<S>>,
        false_branch: Box<Expr<S>>,
        tag: S::If,
    },
    Closure {
        arguments: Vec<S::ClosureArgument>,
        body: Box<Expr<S>>,
        tag: S::Closure,
    },
}

#[derive(Clone)]
pub struct PackField<S: Stage> {
    pub name: String,
    pub value: Expr<S>,
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
    pub fn from_struct(struct_def: Struct<S>) -> Self {
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

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Type::Named { name, arguments },
                Type::Named {
                    name: name1,
                    arguments: arguments1,
                },
            ) => name == name1 && arguments == arguments1,
            (Type::Generic { name }, Type::Generic { name: name1 }) => name == name1,
            (
                Type::Function { arguments, result },
                Type::Function {
                    arguments: arguments1,
                    result: result1,
                },
            ) => arguments == arguments1 && result.as_ref() == result1.as_ref(),
            (
                Type::Unification { name, value },
                Type::Unification {
                    name: name1,
                    value: value1,
                },
            ) => name == name1 && value.get() == value1.get(),
            _ => false,
        }
    }
}

impl Eq for Type {}

impl hash::Hash for Type {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Type::Named { name, arguments } => {
                name.hash(state);
                arguments.hash(state);
            }
            Type::Generic { name } => {
                name.hash(state);
            }
            Type::Function { arguments, result } => {
                arguments.hash(state);
                result.as_ref().hash(state);
            }
            Type::Unification { name, value } => {
                name.hash(state);
                value.get().hash(state);
            }
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

    pub fn unification(name: String) -> Self {
        Self::Unification {
            name,
            value: Rc::new(OnceCell::default()),
        }
    }

    pub fn unwrap<'a>(name: &String, value: &'a OnceCell<Type>) -> &'a Type {
        value
            .get()
            .expect(&format!("undefined unification variable {}", name))
    }

    pub fn canonical(&self) -> &Type {
        if let Self::Unification { value, .. } = self {
            value.get().unwrap_or(self)
        } else {
            self
        }
    }
}

impl<S: DisplayStage> fmt::Display for Program<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.structs {
            writeln!(f, "{}", s)?;
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
            Expr::Primitive {
                primitive,
                arguments,
            } => match primitive {
                Primitive::Add => {
                    write!(f, "({} + {})", &arguments[0], &arguments[1])
                }
                Primitive::Sub => {
                    write!(f, "({} - {})", &arguments[0], &arguments[1])
                }
                Primitive::Mul => {
                    write!(f, "({} * {})", &arguments[0], &arguments[1])
                }
            },
            Expr::StructPack { name, fields, tag } => {
                write!(f, "{}[{}]", name, tag)?;
                let mut strukt = f.debug_struct("");
                for field in fields {
                    strukt.field(&field.name, &field.value);
                }
                strukt.finish()
            }
            Expr::If {
                predicate,
                true_branch,
                false_branch,
                ..
            } => write!(
                f,
                "if {} then {} else {}",
                predicate, true_branch, false_branch
            ),
            Expr::Closure {
                arguments,
                body,
                tag,
            } => {
                if arguments.len() == 1 {
                    write!(f, "{:?}", arguments[0])?;
                } else {
                    let mut args = f.debug_tuple("");
                    for arg in arguments {
                        args.field(arg);
                    }
                    args.finish()?;
                }
                write!(f, " [{tag}] => {}", body.as_ref())
            }
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
            Literal::Boolean(boolean) => write!(f, "{}", boolean),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Primitive::Add => "+",
            Primitive::Sub => "-",
            Primitive::Mul => "*",
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

impl<S: DisplayStage> fmt::Display for Struct<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct(&format!("struct {}", self.name));
        for field in &self.fields {
            s.field(&field.name, &field.typ);
        }
        s.finish()?;
        write!(f, " {}", self.tag)
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
            Type::Unification { name, value } => {
                if let Some(value) = value.get() {
                    write!(f, "{value:?}")
                } else {
                    write!(f, "{name}?")
                }
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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
