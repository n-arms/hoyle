use crate::check;
use crate::env::*;
use crate::specialize::{apply, specialize_arguments};
use im::HashMap;
use im::HashSet;
use tree::parsed;
use tree::typed::*;
use tree::String;
use unzip3::Unzip3;

pub fn program(program: &parsed::Program) -> Result<Program> {
    let struct_signatures = program
        .structs
        .iter()
        .map(|strukt| {
            let fields = strukt
                .fields
                .iter()
                .map(|field| (field.name.clone(), field.typ.clone()))
                .collect();
            (
                strukt.name.clone(),
                StructScheme {
                    fields,
                    result: Type::Named {
                        name: strukt.name.clone(),
                        arguments: Vec::new(),
                    },
                },
            )
        })
        .collect();
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

    let env = Env::new(
        HashMap::new(),
        functions_signatures,
        HashSet::new(),
        struct_signatures,
    );

    let functions = program
        .functions
        .iter()
        .map(|func| function(env.clone(), func))
        .collect::<Result<_>>()?;

    let structs = program
        .structs
        .iter()
        .map(|to_infer| strukt(to_infer))
        .collect();

    Ok(Program { structs, functions })
}

fn strukt(to_infer: &parsed::Struct) -> Struct {
    Struct {
        name: to_infer.name.clone(),
        fields: to_infer.fields.clone(),
        tag: (),
    }
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
            ..
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
        parsed::Expr::StructPack { name, fields, .. } => {
            let scheme = env.lookup_struct(&name)?;
            let res = fields
                .iter()
                .map(|field| {
                    let want = scheme.fields.get(&field.name).unwrap().clone();
                    let typed = expr(env, &field.value)?;
                    let field = PackField {
                        name: field.name.clone(),
                        value: typed.clone(),
                    };
                    Ok((want, typed, field))
                })
                .collect::<Result<Vec<_>>>()?;
            let (want, got, fields): (Vec<_>, Vec<_>, Vec<_>) = res.into_iter().unzip3();
            let spec = specialize_arguments(env, &want, &got)?;
            assert!(spec.is_empty());
            let result = apply(&scheme.result, &spec)?;
            Ok(Expr::StructPack {
                name: name.clone(),
                fields,
                tag: StructPack {
                    result,
                    generics: Vec::new(),
                },
            })
        }
        parsed::Expr::If {
            predicate,
            true_branch,
            false_branch,
            tag,
        } => {
            let typed_predicate = expr(env, &predicate)?;
            let typed_true = expr(env, &true_branch)?;
            let typed_false = expr(env, &false_branch)?;
            if typed_predicate.get_type() != Type::bool() {
                return Err(Error::TypeMismatch {
                    expected: Type::bool(),
                    got: typed_predicate.get_type(),
                });
            }
            if typed_true.get_type() != typed_false.get_type() {
                return Err(Error::TypeMismatch {
                    expected: typed_true.get_type(),
                    got: typed_false.get_type(),
                });
            }
            Ok(Expr::If {
                predicate: Box::new(typed_predicate),
                true_branch: Box::new(typed_true),
                false_branch: Box::new(typed_false),
                tag: *tag,
            })
        }
        parsed::Expr::Closure {
            arguments, body, ..
        } => {
            let captures = {
                let vars = free_variables(body.as_ref());
                vars.into_iter()
                    .map(|name| {
                        let typ = env.lookup_variable(&name)?;
                        Ok(Argument { name, typ })
                    })
                    .collect::<Result<Vec<_>>>()?
            };
            let typed_body = {
                let mut inner_env = env.clone();
                for arg in arguments {
                    inner_env.define_variable(arg.name.clone(), arg.typ.clone());
                }
                expr(&inner_env, body.as_ref())?
            };

            let tag = Closure {
                captures,
                result: Type::Function {
                    arguments: arguments.iter().map(|arg| arg.typ.clone()).collect(),
                    result: Box::new(typed_body.get_type()),
                },
            };
            Ok(Expr::Closure {
                arguments: arguments.clone(),
                body: Box::new(typed_body),
                tag,
            })
        }
    }
}

fn free_variables(expr: &parsed::Expr) -> HashSet<String> {
    match expr {
        parsed::Expr::Variable { name, .. } => HashSet::unit(name.clone()),
        parsed::Expr::Literal { .. } => HashSet::new(),
        parsed::Expr::CallDirect { arguments, .. } => {
            arguments.iter().flat_map(free_variables).collect()
        }
        parsed::Expr::Primitive { arguments, .. } => {
            arguments.iter().flat_map(free_variables).collect()
        }
        parsed::Expr::Block(block) => free_variables_block(block),
        parsed::Expr::StructPack { fields, .. } => fields
            .iter()
            .flat_map(|field| free_variables(&field.value))
            .collect(),
        parsed::Expr::If {
            predicate,
            true_branch,
            false_branch,
            ..
        } => free_variables(&predicate)
            .union(free_variables(&true_branch))
            .union(free_variables(&false_branch)),
        parsed::Expr::Closure {
            arguments, body, ..
        } => {
            let without = arguments.iter().map(|arg| arg.name.clone()).collect();
            free_variables(&body).relative_complement(without)
        }
    }
}

fn free_variables_block(block: &parsed::Block) -> HashSet<String> {
    let mut without = HashSet::new();
    let mut free = HashSet::new();
    for stmt in &block.stmts {
        match stmt {
            parsed::Statement::Let { name, value, .. } => {
                free.extend(free_variables(value).relative_complement(without.clone()));
                without.insert(name.clone());
            }
        }
    }
    free
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
