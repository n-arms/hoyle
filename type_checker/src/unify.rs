use crate::env::{Error, Result};
use tree::typed::Type;

pub fn unify(expected: &Type, found: &Type) -> Result<()> {
    match (expected, found) {
        (
            Type::Named {
                name: expected_name,
                arguments: expected_args,
            },
            Type::Named {
                name: found_name,
                arguments: found_args,
            },
        ) => {
            if expected_name != found_name {
                return Err(Error::NamedTypeMismatch {
                    expected: expected_name.clone(),
                    got: found_name.clone(),
                });
            }
            for (expected, found) in expected_args.iter().zip(found_args) {
                unify(expected, found)?;
            }
            Ok(())
        }

        (Type::Unification { value, .. }, _) => {
            if let Some(inner) = value.get() {
                unify(inner, found)
            } else {
                value.set(found.clone()).unwrap();
                Ok(())
            }
        }
        (_, Type::Unification { value, .. }) => {
            if let Some(inner) = value.get() {
                unify(expected, inner)
            } else {
                value.set(expected.clone()).unwrap();
                Ok(())
            }
        }
        (
            Type::Function {
                arguments: expected_args,
                result: expected_res,
            },
            Type::Function {
                arguments: found_args,
                result: found_res,
            },
        ) => {
            for (expected, found) in expected_args.iter().zip(found_args) {
                unify(expected, found)?;
            }
            unify(&expected_res, &found_res)
        }
        (Type::Generic { name: expected }, Type::Generic { name: found }) => {
            if expected == found {
                Ok(())
            } else {
                Err(Error::NamedTypeMismatch {
                    expected: expected.clone(),
                    got: found.clone(),
                })
            }
        }
        _ => Err(Error::TypeMismatch {
            expected: expected.clone(),
            got: found.clone(),
        }),
    }
}
