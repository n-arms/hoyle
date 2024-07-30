use std::result;

use im::{HashMap, HashSet};
use tree::{typed::*, String};

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    UnknownVariable(String),
    UnknownFunction(String),
    NamedTypeMismatch {
        expected: String,
        got: String,
    },
    TypeMismatch {
        expected: Type,
        got: Type,
    },
    GenericTypeMismatch {
        name: String,
        first: Type,
        second: Type,
    },
    UnspecifiedGeneric {
        generic: Generic,
    },
    UnknownStruct(String),
}

#[derive(Clone)]
pub struct FunctionScheme {
    pub generics: Vec<Generic>,
    pub arguments: Vec<Type>,
    pub result: Type,
}

#[derive(Clone)]
pub struct StructScheme {
    pub fields: HashMap<String, Type>,
    pub result: Type,
}

#[derive(Clone)]
pub struct Env {
    variables: HashMap<String, Type>,
    functions: HashMap<String, FunctionScheme>,
    generics: HashSet<String>,
    structs: HashMap<String, StructScheme>,
}

impl Env {
    pub fn new(
        variables: HashMap<String, Type>,
        functions: HashMap<String, FunctionScheme>,
        generics: HashSet<String>,
        structs: HashMap<String, StructScheme>,
    ) -> Self {
        Self {
            variables,
            functions,
            generics,
            structs,
        }
    }
    pub fn define_generics<'a>(&mut self, generics: impl Iterator<Item = &'a Generic>) {
        self.generics
            .extend(generics.map(|generic| generic.name.clone()))
    }

    pub fn define_arguments<'a>(&mut self, arguments: impl Iterator<Item = &'a Argument>) {
        self.variables
            .extend(arguments.map(|arg| (arg.name.clone(), arg.typ.clone())));
    }

    pub fn lookup_variable(&self, name: &String) -> Result<Type> {
        self.variables
            .get(name)
            .ok_or(Error::UnknownVariable(name.clone()))
            .cloned()
    }

    pub fn lookup_function(&self, name: &String) -> Result<FunctionScheme> {
        self.functions
            .get(name)
            .ok_or(Error::UnknownFunction(name.clone()))
            .cloned()
    }

    pub fn define_variable(&mut self, name: String, typ: Type) {
        self.variables.insert(name, typ);
    }

    pub fn lookup_struct(&self, name: &String) -> Result<StructScheme> {
        self.structs
            .get(name)
            .ok_or(Error::UnknownStruct(name.clone()))
            .cloned()
    }
}
