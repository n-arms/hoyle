use crate::env::*;
use im::HashMap;
use im::HashSet;
use tree::parsed;
use tree::typed::*;
use tree::String;

pub type Spec = HashMap<String, Type>;

pub fn union(first: Spec, second: Spec) -> Result<Spec> {
    let errors = first
        .clone()
        .intersection_with_key(second.clone(), |name, old_typ, typ| {
            Error::GenericTypeMismatch {
                name: name.clone(),
                first: old_typ,
                second: typ,
            }
        });
    if !errors.is_empty() {
        Ok(first.union(second))
    } else {
        Err(errors.values().next().unwrap().clone())
    }
}

pub fn specialize_arguments(
    env: &Env,
    general: &[Type],
    specific: &[Expr],
) -> Result<HashMap<String, Type>> {
    let arg_specs = general
        .into_iter()
        .zip(specific)
        .map(|(typ, expr)| specialize(env, typ, &expr.get_type()));
    let mut spec = HashMap::new();
    for arg_spec in arg_specs {
        spec = union(spec, arg_spec?)?;
    }
    Ok(spec)
}

fn specialize(env: &Env, general: &Type, specific: &Type) -> Result<HashMap<String, Type>> {
    match (general, specific) {
        (
            Type::Named { name, arguments },
            Type::Named {
                name: newName,
                arguments: newArguments,
            },
        ) => {
            if name != newName {
                Err(Error::NamedTypeMismatch {
                    expected: name.clone(),
                    got: newName.clone(),
                })
            } else {
                let mut spec = HashMap::new();
                for (old, new) in arguments.iter().zip(newArguments) {
                    spec = union(spec, specialize(env, old, new)?)?;
                }
                Ok(spec)
            }
        }
        (Type::Generic { name }, typ) => Ok(HashMap::unit(name.clone(), typ.clone())),
        (
            Type::Function { arguments, result },
            Type::Function {
                arguments: newArguments,
                result: newResult,
            },
        ) => {
            let mut spec = specialize(env, &result, &newResult)?;
            for (old, new) in arguments.iter().zip(newArguments) {
                spec = union(spec, specialize(env, old, new)?)?;
            }
            Ok(spec)
        }
        (expected, got) => Err(Error::TypeMismatch {
            expected: expected.clone(),
            got: got.clone(),
        }),
    }
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
    }
}
