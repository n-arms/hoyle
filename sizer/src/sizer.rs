use im::hashmap;
use im::HashMap;
use tree::sized::*;
use tree::type_passing;
use tree::String;

use crate::env::Env;
use crate::env::StructInstance;

pub fn program(
    to_size: &type_passing::Program,
    structs: &type_passing::StructBuilders,
) -> (Program, StructBuilders) {
    let mut env = Env::default();
    let mut builders = StructBuilders::default();
    for (name, to_size) in structs.iter() {
        strukt(&env, &mut builders, name.clone(), to_size);
    }
    for strukt in &to_size.structs {
        env.define_struct(strukt.name.clone(), strukt.clone());
    }
    let functions = to_size
        .functions
        .iter()
        .map(|func| function(&env, func))
        .collect();
    let program = Program {
        structs: to_size.structs.clone(),
        functions,
    };
    (program, builders)
}

fn strukt(
    env: &Env,
    builders: &mut StructBuilders,
    name: String,
    to_size: &type_passing::StructBuilder,
) {
    let sized_args = to_size
        .arguments
        .iter()
        .map(|arg| Argument {
            name: arg.name.clone(),
            typ: arg.typ.clone(),
            witness: Witness::Type,
        })
        .collect();
    let sized_fields = to_size
        .fields
        .iter()
        .map(|field| expr(env, field))
        .collect();
    let builder = StructBuilder {
        arguments: sized_args,
        fields: sized_fields,
    };
    builders.define_struct(name, builder);
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
                    "Bool" => 1,
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
        Type::Function { arguments, result } => todo!(),
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
                result: Type::Named {
                    name: String::from("Type"),
                    arguments: Vec::new(),
                },
                witness: Witness::Type,
            },
        }),
    }
}
