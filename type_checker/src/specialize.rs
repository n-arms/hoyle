use std::cell::OnceCell;
use std::rc::Rc;

use crate::env::*;
use im::HashMap;
use tree::typed::*;
use tree::String;

pub type Spec = HashMap<String, Type>;

pub fn make_specialization(generics: &[Generic]) -> Spec {
    generics
        .iter()
        .map(|generic| {
            (
                generic.name.clone(),
                Type::Unification {
                    name: generic.name.clone(),
                    value: Rc::new(OnceCell::default()),
                },
            )
        })
        .collect()
}

pub fn apply(typ: &Type, spec: &Spec) -> Result<Type> {
    match typ {
        Type::Named { name, arguments } => Ok(Type::Named {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|arg| apply(arg, spec))
                .collect::<Result<_>>()?,
        }),
        Type::Generic { name } => {
            if let Some(typ) = spec.get(name) {
                Ok(typ.clone())
            } else {
                Ok(typ.clone())
            }
        }
        Type::Function { arguments, result } => Ok(Type::Function {
            arguments: arguments
                .iter()
                .map(|arg| apply(arg, spec))
                .collect::<Result<_>>()?,
            result: Box::new(apply(result, spec)?),
        }),
        Type::Unification { name, value } => apply(Type::unwrap(name, value), spec),
    }
}
