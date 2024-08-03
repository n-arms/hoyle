use crate::env::*;
use crate::infer;
use crate::infer::closure_captures;
use crate::unify::unify;
use tree::parsed;
use tree::typed::*;

pub fn expr(env: &Env, to_check: &parsed::Expr, want: &Type) -> Result<Expr> {
    match (to_check, want.canonical()) {
        (
            parsed::Expr::Closure {
                arguments, body, ..
            },
            Type::Function {
                arguments: arg_types,
                result,
            },
        ) => {
            let mut inner_env = env.clone();
            let typed_arguments = arg_types
                .iter()
                .zip(arguments)
                .map(|(typ, arg)| {
                    if let Some(annotated) = &arg.typ {
                        unify(typ, annotated)?;
                    }
                    inner_env.define_variable(arg.name.clone(), typ.clone());
                    Ok(ClosureArgument {
                        name: arg.name.clone(),
                        typ: typ.clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let typed_body = expr(&inner_env, &body, &result)?;

            let captures = closure_captures(env, &arguments, body.as_ref())?;
            let tag = Closure {
                captures,
                result: Type::Function {
                    arguments: typed_arguments.iter().map(|arg| arg.typ.clone()).collect(),
                    result: Box::new(typed_body.get_type()),
                },
            };

            Ok(Expr::Closure {
                arguments: typed_arguments,
                body: Box::new(typed_body),
                tag,
            })
        }
        _ => {
            let typed = infer::expr(env, to_check)?;
            unify(want, &typed.get_type())?;
            Ok(typed)
        }
    }
}
