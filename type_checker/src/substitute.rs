use crate::env::*;
use crate::specialize::Spec;
use im::HashMap;
use im::HashSet;
use tree::parsed;
use tree::typed::*;

pub fn apply(typ: &Type, spec: &Spec) -> Type {
    match typ {
        Type::Named { .. } => typ.clone(),
        Type::Generic { name } => spec.get(name),
        Type::Function { arguments, result } => Type::Function {
            arguments: arguments.into_iter().map(|typ| apply(typ, spec)).collect(),
            result: Box::new(apply(result, spec)),
        },
    }
}
