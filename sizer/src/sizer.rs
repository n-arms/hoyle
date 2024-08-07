use tree::sized::*;
use tree::type_passing;

use crate::env::Env;
use crate::env::StructInstance;

pub fn program(to_size: &type_passing::Program) -> Program {
    let mut env = Env::default();

    let structs = to_size
        .structs
        .iter()
        .map(|to_size| strukt(&mut env, to_size))
        .collect();
    let functions = to_size
        .functions
        .iter()
        .map(|func| function(&env, func))
        .collect();
    Program { structs, functions }
}

fn strukt(env: &mut Env, to_size: &type_passing::Struct) -> Struct {
    let arguments = to_size
        .tag
        .arguments
        .iter()
        .map(|arg| Variable {
            name: arg.clone(),
            witness: Witness::Type,
        })
        .collect();
    let fields = to_size
        .tag
        .fields
        .iter()
        .map(|field| expr(env, field))
        .collect();
    let tag = StructMeta { arguments, fields };
    let sized = Struct {
        name: to_size.name.clone(),
        fields: to_size.fields.clone(),
        tag,
    };
    env.define_struct(to_size.name.clone(), sized.clone());
    sized
}

fn function(env: &Env, to_size: &type_passing::Function) -> Function {
    let mut env = env.clone();
    let args = to_size
        .arguments
        .iter()
        .map(|arg| {
            let witness = type_witness(&env, &arg.typ);
            env.define_variable(arg.name.clone(), witness.clone());
            Argument {
                name: arg.name.clone(),
                typ: arg.typ.clone(),
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
                    witness: type_witness(env, &tag.result),
                    signature: tag.signature.clone(),
                },
            }
        }
        type_passing::Expr::Block(to_size) => Expr::Block(block(env, to_size)),
        type_passing::Expr::Primitive {
            primitive,
            arguments,
        } => {
            let sized_args = arguments.iter().map(|arg| expr(env, arg)).collect();
            Expr::Primitive {
                primitive: *primitive,
                arguments: sized_args,
            }
        }
        type_passing::Expr::StructPack { name, fields, tag } => {
            let witness = type_witness(env, &tag.result);
            let sized_fields = fields
                .iter()
                .map(|to_size| PackField {
                    name: to_size.name.clone(),
                    value: expr(env, &to_size.value),
                })
                .collect();
            Expr::StructPack {
                name: name.clone(),
                fields: sized_fields,
                tag: StructPack {
                    result: tag.result.clone(),
                    witness,
                },
            }
        }
        type_passing::Expr::If {
            predicate,
            true_branch,
            false_branch,
            ..
        } => {
            let sized_predicate = expr(env, &predicate);
            let sized_true = expr(env, &true_branch);
            let sized_false = expr(env, &false_branch);
            Expr::If {
                tag: If {
                    witness: type_witness(env, &sized_true.get_type()),
                },
                predicate: Box::new(sized_predicate),
                true_branch: Box::new(sized_true),
                false_branch: Box::new(sized_false),
            }
        }
        type_passing::Expr::Closure {
            arguments,
            body,
            tag,
        } => {
            let sized_args: Vec<_> = arguments
                .iter()
                .map(|arg| {
                    let witness = type_witness(env, &arg.typ);
                    ClosureArgument {
                        name: arg.name.clone(),
                        typ: arg.typ.clone(),
                        witness,
                    }
                })
                .collect();
            let sized_body = {
                let mut inner_env = env.clone();
                for arg in &sized_args {
                    inner_env.define_variable(arg.name.clone(), arg.witness.clone());
                }
                expr(&inner_env, &body)
            };
            let value_captures = tag
                .value_captures
                .iter()
                .map(|arg| {
                    let witness = type_witness(env, &arg.typ);
                    ClosureArgument {
                        name: arg.name.clone(),
                        typ: arg.typ.clone(),
                        witness,
                    }
                })
                .collect();
            let type_captures = tag
                .type_captures
                .iter()
                .map(|arg| {
                    let witness = type_witness(env, &arg.typ);
                    ClosureArgument {
                        name: arg.name.clone(),
                        typ: arg.typ.clone(),
                        witness,
                    }
                })
                .collect();
            let env_struct = {
                let builder_args = tag
                    .env
                    .tag
                    .arguments
                    .iter()
                    .map(|arg| Variable {
                        name: arg.clone(),
                        witness: Witness::Type,
                    })
                    .collect();
                let builder_fields = tag
                    .env
                    .tag
                    .fields
                    .iter()
                    .map(|field| expr(env, &field))
                    .collect();
                Struct {
                    name: tag.env.name.clone(),
                    fields: tag.env.fields.clone(),
                    tag: StructMeta {
                        arguments: builder_args,
                        fields: builder_fields,
                    },
                }
            };
            let tag = Closure {
                value_captures,
                type_captures,
                result: tag.result.clone(),
                witness: Witness::closure(),
                env: env_struct,
            };
            Expr::Closure {
                arguments: sized_args,
                body: Box::new(sized_body),
                tag,
            }
        }
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
                env.define_variable(name.clone(), type_witness(&env, let_type));
                Statement::Let {
                    name: Variable {
                        name: name.clone(),
                        witness: type_witness(&env, &let_type),
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

fn type_witness(env: &Env, to_witness: &Type) -> Witness {
    match to_witness {
        Type::Named { name, arguments } => {
            if !arguments.is_empty() {
                unimplemented!("argument types");
            }
            Witness::Trivial {
                size: match name.as_str() {
                    "F64" => 8,
                    "Bool" => 8,
                    "Type" => return Witness::Type,
                    _ => return struct_witness(env, &env.lookup_struct(&name)),
                },
            }
        }
        Type::Generic { name } => Witness::Dynamic {
            value: Box::new(Expr::Variable {
                name: Variable {
                    name: name.clone(),
                    witness: Witness::Type,
                },
                typ: Type::typ(),
            }),
        },
        Type::Function { .. } => Witness::closure(),
        Type::Unification { name, value } => type_witness(env, Type::unwrap(name, value)),
    }
}

fn struct_witness(env: &Env, to_witness: &Struct) -> Witness {
    env.witness_struct_instance(
        StructInstance {
            name: to_witness.name.clone(),
        },
        (),
    );
    Witness::Dynamic {
        value: Box::new(Expr::CallDirect {
            function: to_witness.name.clone(),
            arguments: Vec::new(),
            tag: Call {
                result: Type::typ(),
                witness: Witness::Type,
                signature: vec![Convention::Out],
            },
        }),
    }
}
