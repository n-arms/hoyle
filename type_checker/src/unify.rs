use crate::error::{Error, Result};
use ir::qualified;
use ir::typed::Type;

pub fn types<'expr, 'ident>(
    to_check: Type<'expr, 'ident>,
    target: Type<'expr, 'ident>,
) -> Result<'expr, 'ident, ()> {
    match (to_check, target) {
        (
            Type::Named { name, .. },
            Type::Named {
                name: target_name, ..
            },
        ) => {
            if name == target_name {
                Ok(())
            } else {
                Err(Error::TypeMismatch {
                    expected: target,
                    found: to_check,
                })
            }
        }
        (
            Type::Arrow {
                arguments,
                return_type,
                ..
            },
            Type::Arrow {
                arguments: target_arguments,
                return_type: target_return_type,
                ..
            },
        ) => {
            for (arg, target_arg) in arguments.iter().zip(target_arguments) {
                types(*arg, *target_arg)?;
            }
            types(*return_type, *target_return_type)
        }
        _ => todo!(),
    }
}
