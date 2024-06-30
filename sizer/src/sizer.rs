use im::hashmap;
use im::HashMap;
use tree::sized::*;
use tree::type_passing;
use tree::String;

use crate::env::Env;

pub fn program(to_size: &type_passing::Program) -> Program {
    let env = Env::default();
    let functions = to_size
        .functions
        .iter()
        .map(|func| function(&env, func))
        .collect();
    Program {
        structs: to_size.structs.clone(),
        functions,
    }
}

fn function(env: &Env, to_size: &type_passing::Function) -> Function {
    let mut env = env.clone();
    let args = to_size
        .arguments
        .iter()
        .map(|arg| {
            let size = typ(&env, &arg.typ);
            let witness = type_witness(&env, &arg.typ);
            env.define_variable(arg.name.clone(), size.clone(), witness.clone());
            Argument {
                name: arg.name.clone(),
                typ: arg.typ.clone(),
                size,
                witness,
            }
        })
        .collect();
    let body = expr(&env, &to_size.body);
    Function {
        name: to_size.name.clone(),
        generics: to_size.generics.clone(),
        arguments: args,
        result: to_size.result.clone(),
        body,
    }
}

fn expr(env: &Env, to_size: &type_passing::Expr) -> Expr {
    match to_size {
        type_passing::Expr::Variable { name, typ } => {
            let var = env.lookup_variable(name);
            Expr::Variable {
                name: var,
                typ: typ.clone(),
            }
        }
        type_passing::Expr::Literal { literal } => Expr::Literal {
            literal: literal.clone(),
        },
        type_passing::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let sized_args = arguments.iter().map(|arg| expr(env, arg)).collect();
            Expr::CallDirect {
                function: function.clone(),
                arguments: sized_args,
                tag: Call {
                    result: tag.result.clone(),
                    size: typ(env, &tag.result),
                    witness: Some(Box::new(type_witness(env, &tag.result))),
                },
            }
        }
        type_passing::Expr::Block(to_size) => Expr::Block(block(env, to_size)),
    }
}

fn block(env: &Env, to_size: &type_passing::Block) -> Block {
    let mut env = env.clone();
    let stmts = to_size
        .stmts
        .iter()
        .map(|stmt| match stmt {
            type_passing::Statement::Let {
                name,
                typ: let_type,
                value,
            } => {
                let size = typ(&env, let_type);
                env.define_variable(name.clone(), size.clone(), type_witness(&env, let_type));
                Statement::Let {
                    name: Variable {
                        name: name.clone(),
                        size,
                        witness: Box::new(type_witness(&env, &let_type)),
                    },
                    typ: let_type.clone(),
                    value: expr(&env, value),
                }
            }
        })
        .collect();
    Block {
        stmts,
        result: Box::new(expr(&env, &to_size.result)),
    }
}

fn typ(env: &Env, to_size: &Type) -> Size {
    match to_size {
        Type::Named { name, arguments } => {
            let sizes = hashmap![
                String::from("Type") => 24,
                String::from("F64") => 8,
                String::from("Bool") => 8,
            ];
            if let Some(size) = sizes.get(name) {
                return Size {
                    static_size: *size,
                    dynamic: Vec::new(),
                };
            }
            let struct_def = env.lookup_struct(name);
            if !arguments.is_empty() {
                todo!()
            }
            let mut total_size = Size {
                static_size: 0,
                dynamic: Vec::new(),
            };
            for field in struct_def.fields {
                let size = typ(env, &field.typ);
                total_size.static_size += size.static_size;
                total_size.dynamic.extend(size.dynamic);
            }
            total_size
        }
        Type::Generic { name } => Size {
            static_size: 0,
            dynamic: vec![Expr::Variable {
                name: Variable {
                    name: name.clone(),
                    size: Size::new_static(24),
                    witness: Box::new(type_witness_table()),
                },
                typ: Type::typ(),
            }],
        },
        Type::Function { .. } => Size::new_static(16),
    }
}

fn type_witness_table() -> Expr {
    Expr::CallDirect {
        function: String::from("Type"),
        arguments: Vec::new(),
        tag: Call {
            result: Type::typ(),
            size: Size::new_static(24),
            witness: None,
        },
    }
}

fn type_witness(env: &Env, to_witness: &Type) -> Expr {
    match to_witness {
        Type::Named { name, arguments } => Expr::CallDirect {
            function: name.clone(),
            arguments: arguments
                .iter()
                .map(|to_witness| type_witness(env, to_witness))
                .collect(),
            tag: Call {
                result: Type::typ(),
                size: Size::new_static(24),
                witness: Some(Box::new(type_witness_table())),
            },
        },
        Type::Generic { name } => Expr::Variable {
            name: Variable {
                name: name.clone(),
                size: Size::new_static(24),
                witness: Box::new(type_witness_table()),
            },
            typ: Type::typ(),
        },
        Type::Function { arguments, result } => todo!(),
    }
}
