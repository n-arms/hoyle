use crate::check;
use crate::env::*;
use crate::specialize::{apply, specialize_arguments};
use im::HashMap;
use im::HashSet;
use tree::parsed;
use tree::typed::*;

pub fn program(program: &parsed::Program) -> Result<Program> {
    let functions_signatures = program
        .functions
        .iter()
        .map(|func| {
            (
                func.name.clone(),
                FunctionScheme {
                    generics: func.generics.clone(),
                    arguments: func.arguments.iter().map(|arg| arg.typ.clone()).collect(),
                    result: func.result.clone(),
                },
            )
        })
        .collect();

    let env = Env::new(HashMap::new(), functions_signatures, HashSet::new());

    let functions = program
        .functions
        .iter()
        .map(|func| function(env.clone(), func))
        .collect::<Result<_>>()?;
    Ok(Program {
        structs: program.structs.clone(),
        functions,
    })
}

pub fn function(mut env: Env, function: &parsed::Function) -> Result<Function> {
    env.define_generics(function.generics.iter());
    env.define_arguments(function.arguments.iter());
    let body = check::expr(&env, &function.body, &function.result)?;
    Ok(Function {
        name: function.name.clone(),
        generics: function.generics.clone(),
        arguments: function.arguments.clone(),
        result: function.result.clone(),
        body,
    })
}

pub fn expr(env: &Env, to_infer: &parsed::Expr) -> Result<Expr> {
    match to_infer {
        parsed::Expr::Variable { name, .. } => {
            let typ = env.lookup_variable(name)?;
            Ok(Expr::Variable {
                name: name.clone(),
                typ,
            })
        }
        parsed::Expr::Literal { literal } => Ok(Expr::Literal {
            literal: literal.clone(),
        }),
        parsed::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let scheme = env.lookup_function(function)?;
            let typed_arguments = arguments
                .into_iter()
                .map(|arg| expr(env, arg))
                .collect::<Result<Vec<_>>>()?;

            let spec = specialize_arguments(env, &scheme.arguments, &typed_arguments)?;
            let result = apply(&scheme.result, &spec)?;
            let generics = scheme
                .generics
                .into_iter()
                .map(|generic| {
                    spec.get(&generic.name)
                        .ok_or(Error::UnspecifiedGeneric { generic })
                        .cloned()
                })
                .collect::<Result<_>>()?;

            Ok(Expr::CallDirect {
                function: function.clone(),
                arguments: typed_arguments,
                tag: Call { result, generics },
            })
        }
        parsed::Expr::Block(b) => {
            let typed_block = block(env, b)?;
            Ok(Expr::Block(typed_block))
        }
        parsed::Expr::Primitive {
            primitive,
            arguments,
        } => {
            let typed_arguments = arguments
                .into_iter()
                .map(|arg| expr(env, arg))
                .collect::<Result<Vec<_>>>()?;
            Ok(Expr::Primitive {
                primitive: *primitive,
                arguments: typed_arguments,
            })
        }
    }
}

fn block(env: &Env, block: &parsed::Block) -> Result<Block> {
    let mut env = env.clone();
    let typed_stmts = block
        .stmts
        .iter()
        .map(|stmt: &parsed::Statement| match stmt {
            parsed::Statement::Let { name, value, .. } => {
                let typed_value = expr(&env, value)?;
                env.define_variable(name.clone(), typed_value.get_type());
                Ok(Statement::Let {
                    name: name.clone(),
                    typ: typed_value.get_type(),
                    value: typed_value,
                })
            }
        })
        .collect::<Result<_>>()?;
    let typed_result = expr(&env, &block.result)?;
    Ok(Block {
        stmts: typed_stmts,
        result: Box::new(typed_result),
    })
}
