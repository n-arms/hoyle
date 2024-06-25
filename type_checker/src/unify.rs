/*
use crate::env::is_unification;
use crate::error::{Error, Result};
use crate::substitute::Substitution;
use ir::qualified::Type;

pub fn check_types<'expr>(to_check: Type<'expr>, target: Type<'expr>) -> Result<'expr, ()> {
    let sub = substitute_types(to_check, target)?;

    if sub.is_empty() {
        Ok(())
    } else {
        todo!()
    }
}

pub fn substitute_types<'expr>(
    general: Type<'expr>,
    specific: Type<'expr>,
) -> Result<'expr, Substitution<'expr>> {
    match (general, specific) {
        (
            Type::Function {
                arguments,
                return_type,
                ..
            },
            Type::Function {
                arguments: specific_arguments,
                return_type: specific_return_type,
                ..
            },
        ) => {
            let mut sub = substitute_types(*return_type, *specific_return_type)?;
            for (arg, specific_arg) in arguments.iter().zip(specific_arguments.iter()) {
                sub.union(&substitute_types(*arg, *specific_arg)?);
            }
            Ok(sub)
        }
        (
            Type::Named { name, .. },
            Type::Named {
                name: specific_name,
                ..
            },
        ) if name == specific_name => Ok(Substitution::default()),
        (Type::Named { name, .. }, specific) if is_unification(name) => {
            Ok(Substitution::unit(name, specific))
        }
        (general, specific) => Err(Error::TypeMismatch {
            expected: general,
            found: specific,
        }),
    }
}
*/
