use core::fmt;
use std::cell::RefCell;

use crate::env::Env;
use crate::refcount::count_function;
use ir::bridge::{
    Argument, Block, BuilderArgument, CallArgument, Convention, Expr, Function, Instr, PackField,
    Program, Struct, StructBuilder, Value, Variable, Witness,
};
use tree::sized::{self};
use tree::type_passing::make_signature;
use tree::typed::Type;
use tree::String;

#[derive(Default)]
struct Builder {
    functions: Vec<Function>,
    structs: Vec<Struct>,
}

impl Builder {
    pub fn build(self) -> Program {
        Program {
            structs: self.structs,
            functions: self.functions,
        }
    }
}

pub fn program(to_lower: &sized::Program) -> Program {
    let mut builder = Builder::default();
    for to_lower in &to_lower.structs {
        strukt(to_lower, &mut builder);
    }
    for to_lower in &to_lower.functions {
        let func = function(to_lower, &mut builder);
        builder.functions.push(count_function(func));
    }
    builder.build()
}

pub struct BlockBuilder {
    instrs: RefCell<Vec<Instr>>,
    name: &'static str,
}

impl BlockBuilder {
    pub fn new(name: &'static str) -> Self {
        Self {
            instrs: RefCell::default(),
            name,
        }
    }
}

impl BlockBuilder {
    fn push(&self, instr: Instr) {
        self.instrs.borrow_mut().push(instr);
    }

    fn build(self) -> Block {
        Block {
            instrs: self.instrs.into_inner(),
        }
    }
}

fn strukt(to_lower: &sized::Struct, builder: &mut Builder) {
    let mut env = Env::new();
    let instrs = BlockBuilder::new("struct");
    let fields = to_lower
        .tag
        .fields
        .iter()
        .map(|field| expr(&mut env, field, &instrs, builder))
        .collect();

    let block = instrs.build();
    let lowered_builder = StructBuilder {
        arguments: vec![BuilderArgument {
            name: env.define_variable(String::from("_result"), Type::typ(), Witness::Type),
            convention: Convention::Out,
        }],
        block,
        fields,
        names: env.name_source,
    };

    builder.structs.push(Struct {
        definition: to_lower.clone(),
        builder: lowered_builder,
    });
}

fn function<'a>(to_lower: &sized::Function, builder: &mut Builder) -> Function {
    let mut env = Env::new();
    let body_builder = BlockBuilder::new("body");
    let lowered_arguments: Vec<_> = to_lower
        .arguments
        .iter()
        .cloned()
        .zip(make_signature(to_lower.arguments.len() - 1))
        .map(|(arg, convention)| {
            let arg_witness = witness(&mut env, &arg.witness, &body_builder, builder);
            Argument {
                name: env.define_variable(arg.name, arg.typ, arg_witness),
                convention,
            }
        })
        .collect();
    let result = expr(&mut env, &to_lower.body, &body_builder, builder);
    body_builder.push(Instr::new(
        lowered_arguments[0].name.clone(),
        Expr::mov(result.clone()),
    ));
    let names = env.name_source.clone();
    Function {
        name: to_lower.name.clone(),
        arguments: lowered_arguments,
        body: body_builder.build(),
        names,
    }
}

pub fn expr(
    env: &mut Env,
    to_lower: &sized::Expr,
    instrs: &BlockBuilder,
    builder: &mut Builder,
) -> Variable {
    match to_lower {
        sized::Expr::Variable { name, typ } => {
            let witness = witness(env, &name.witness, instrs, builder);
            env.define_variable(name.name.clone(), typ.clone(), witness)
        }
        sized::Expr::Literal { literal } => {
            let name = env.fresh_name();
            let var = env.define_variable(name, literal.get_type(), Witness::trivial(8));
            instrs.push(Instr::new(var.clone(), Expr::Literal(literal.clone())));
            var
        }
        sized::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let name = env.fresh_name();
            let result_witness = witness(env, &tag.witness, instrs, builder);
            let result = env.define_variable(name, tag.result.clone(), result_witness);
            let mut lowered_arguments: Vec<_> = arguments
                .iter()
                .map(|to_lower| expr(env, to_lower, instrs, builder))
                .collect();
            lowered_arguments.insert(0, result.clone());
            dbg!(&lowered_arguments, &tag.signature.len());
            let tagged_arguments: Vec<_> = lowered_arguments
                .into_iter()
                .zip(tag.signature.clone())
                .map(|(value, convention)| CallArgument {
                    value: Value::Copy(value),
                    convention,
                })
                .collect();
            instrs.push(Instr::new(
                result.clone(),
                Expr::CallDirect {
                    function: function.clone(),
                    arguments: tagged_arguments,
                },
            ));

            result
        }
        sized::Expr::Block(to_lower) => block(env, to_lower, instrs, builder),
        sized::Expr::Primitive {
            primitive,
            arguments,
        } => {
            let name = env.fresh_name();
            let result = env.define_variable(name, arguments[0].get_type(), Witness::trivial(8));
            let lowered_args = arguments
                .iter()
                .map(|to_lower| expr(env, to_lower, instrs, builder))
                .collect();
            instrs.push(Instr::new(
                result.clone(),
                Expr::Primitive(*primitive, lowered_args),
            ));
            result
        }
        sized::Expr::StructPack {
            name: struct_name,
            fields,
            tag,
        } => {
            let name = env.fresh_name();
            let result_witness = witness(env, &tag.witness, instrs, builder);
            let result = env.define_variable(name, tag.result.clone(), result_witness);
            let lowered_fields = fields
                .iter()
                .map(|field| {
                    let value = expr(env, &field.value, instrs, builder);
                    PackField {
                        name: field.name.clone(),
                        value: Value::Copy(value),
                    }
                })
                .collect();
            instrs.push(Instr::new(
                result.clone(),
                Expr::StructPack {
                    name: struct_name.clone(),
                    arguments: lowered_fields,
                },
            ));
            result
        }
        sized::Expr::If {
            predicate,
            true_branch,
            false_branch,
            tag,
        } => {
            let witness = witness(env, &tag.witness, instrs, builder);
            let name = env.fresh_name();
            let result = env.define_variable(name, true_branch.get_type(), witness);
            let lowered_predicate = expr(env, &predicate, instrs, builder);
            let true_instrs = BlockBuilder::new("true branch");
            let lowered_true = expr(env, &true_branch, &true_instrs, builder);
            true_instrs.push(Instr::new(result.clone(), Expr::copy(lowered_true)));
            let false_instrs = BlockBuilder::new("false branch");
            let lowered_false = expr(env, &false_branch, &false_instrs, builder);
            false_instrs.push(Instr::new(result.clone(), Expr::copy(lowered_false)));
            instrs.push(Instr::new(
                result.clone(),
                Expr::If {
                    predicate: lowered_predicate,
                    true_branch: true_instrs.build(),
                    false_branch: false_instrs.build(),
                },
            ));
            result
        }
        sized::Expr::Closure {
            arguments,
            body,
            tag,
        } => {
            let env_name = {
                // closures are compiled into a (function pointer, existential consisting of a heap allocated witness table describing an accompanying env)
                // when you wish to call a closure, you pass all the arguments first, then the environment struct, then the witness table for the environment struct
                // the env structure just holds value captures: generic captures are stuffed into the `extra` field of the witness table like with any other struct
                let mut env_struct = tag.env.clone();
                env_struct.name = struct_name(builder);
                strukt(&env_struct, builder);
                env_struct.name
            };
            let get_value = |name: &String| String::from(format!("{env_name}_get_{name}"));
            let env_type = Type::Named {
                name: env_name.clone(),
                arguments: tag
                    .type_captures
                    .iter()
                    .map(|arg| Type::Generic {
                        name: arg.name.clone(),
                    })
                    .collect(),
            };
            let mocked_function = {
                let function_name = closure_name(&builder);
                let mut arguments = arguments.clone();
                arguments.insert(
                    0,
                    sized::Argument {
                        name: String::from("_result"),
                        typ: body.get_type(),
                        witness: body.get_witness(),
                    },
                );
                let witness_arguments: Vec<_> = tag
                    .type_captures
                    .iter()
                    .map(|arg| sized::Expr::Variable {
                        name: sized::Variable {
                            name: arg.name.clone(),
                            witness: sized::Witness::Type,
                        },
                        typ: sized::Type::typ(),
                    })
                    .collect();
                arguments.push(sized::Argument {
                    name: String::from("_env"),
                    typ: env_type.clone(),
                    witness: sized::Witness::Dynamic {
                        value: Box::new(sized::Expr::CallDirect {
                            tag: sized::Call {
                                result: Type::typ(),
                                witness: sized::Witness::Type,
                                signature: make_signature(witness_arguments.len()),
                            },
                            function: env_name.clone(),
                            arguments: witness_arguments,
                        }),
                    },
                });
                arguments.extend(tag.type_captures.iter().cloned());
                arguments.extend(tag.value_captures.iter().cloned());
                sized::Function {
                    name: function_name,
                    generics: Vec::new(),
                    arguments,
                    result: body.get_type(),
                    body: body.as_ref().clone(),
                }
            };
            let mut lowered_func = function(&mocked_function, builder);
            let real_arguments = lowered_func.arguments.len() - tag.value_captures.len();
            let env_argument = real_arguments - tag.type_captures.len() - 1;
            let env_witness_variable = lowered_func.arguments[env_argument]
                .name
                .witness
                .as_ref()
                .clone()
                .unwrap_dynamic();
            let env_variable = Variable::new(
                String::from("_env"),
                env_type.clone(),
                Witness::Dynamic {
                    location: env_witness_variable.clone(),
                },
            );
            let witness_preamble = BlockBuilder::new("witness preamble");
            let unpacking_type_args: Vec<_> = tag
                .type_captures
                .iter()
                .map(|capture| {
                    Variable::new(capture.name.clone(), capture.typ.clone(), Witness::Type)
                })
                .collect();

            let value_preamble: Vec<_> = tag
                .value_captures
                .iter()
                .map(|arg| {
                    let witness = witness(env, &arg.witness, &witness_preamble, builder);
                    let target = Variable::new(arg.name.clone(), arg.typ.clone(), witness);
                    Instr::new(
                        target,
                        Expr::Unpack {
                            value: env_variable.clone(),
                            field: arg.name.clone(),
                            struct_name: env_name.clone(),
                            type_arguments: unpacking_type_args.clone(),
                        },
                    )
                })
                .collect();
            let preamble = witness_preamble
                .build()
                .instrs
                .into_iter()
                .chain(value_preamble);
            let pre_preamble = first_non_type(&lowered_func.body.instrs);
            for (i, instr) in preamble.enumerate() {
                lowered_func.body.instrs.insert(i + pre_preamble, instr);
            }
            lowered_func.arguments.truncate(real_arguments);
            lowered_func = count_function(lowered_func);

            let made_env_witness = env.fresh_variable(Type::typ(), Witness::Type);
            let made_env = env.fresh_variable(
                env_type,
                Witness::Dynamic {
                    location: made_env_witness.clone(),
                },
            );
            let mut witness_making_args = vec![CallArgument {
                value: Value::Copy(made_env_witness.clone()),
                convention: Convention::Out,
            }];
            witness_making_args.extend(tag.type_captures.iter().map(|capture| {
                let witness = witness(env, &capture.witness, instrs, builder);
                CallArgument {
                    value: Value::Copy(Variable::new(
                        capture.name.clone(),
                        capture.typ.clone(),
                        witness,
                    )),
                    convention: Convention::In,
                }
            }));
            instrs.push(Instr::new(
                made_env_witness.clone(),
                Expr::CallDirect {
                    function: env_name.clone(),
                    arguments: witness_making_args,
                },
            ));
            let env_fields = tag
                .value_captures
                .iter()
                .cloned()
                .map(|capture| {
                    let witness = witness(env, &capture.witness, instrs, builder);
                    PackField {
                        name: capture.name.clone(),
                        value: Value::Copy(Variable::new(capture.name, capture.typ, witness)),
                    }
                })
                .collect();
            instrs.push(Instr::new(
                made_env.clone(),
                Expr::StructPack {
                    name: env_name,
                    arguments: env_fields,
                },
            ));
            let closure_witness = witness(env, &tag.witness, instrs, builder);
            let closure_var = env.fresh_variable(tag.result.clone(), closure_witness);
            instrs.push(Instr::new(
                closure_var.clone(),
                Expr::MakeClosure {
                    function: lowered_func.name.clone(),
                    env: Value::Copy(made_env.clone()),
                    witness: Value::Copy(made_env_witness.clone()),
                },
            ));
            builder.functions.push(lowered_func);
            closure_var
        }
    }
}
fn first_non_type<'a, I: ExactSizeIterator<Item = &'a Instr>>(
    instrs: impl IntoIterator<IntoIter = I>,
) -> usize {
    let iter = instrs.into_iter();
    let length = iter.len();
    iter.enumerate()
        .find_map(|(i, instr)| {
            if let Witness::Type = instr.target.witness.as_ref() {
                None
            } else {
                Some(i)
            }
        })
        .unwrap_or(length)
}

fn block(
    env: &mut Env,
    to_lower: &sized::Block,
    instrs: &BlockBuilder,
    builder: &mut Builder,
) -> Variable {
    for stmt in &to_lower.stmts {
        match stmt {
            sized::Statement::Let { name, typ, value } => {
                let source = expr(env, value, instrs, builder);
                let target =
                    env.define_variable(name.name.clone(), typ.clone(), *source.witness.clone());
                instrs.push(Instr::new(target, Expr::copy(source)));
            }
        }
    }
    expr(env, &to_lower.result, instrs, builder)
}

fn witness(
    env: &mut Env,
    to_lower: &sized::Witness,
    instrs: &BlockBuilder,
    builder: &mut Builder,
) -> Witness {
    match to_lower {
        sized::Witness::Trivial { size } => Witness::Trivial { size: *size },
        sized::Witness::Dynamic { value } => Witness::Dynamic {
            location: expr(env, value.as_ref(), instrs, builder),
        },
        sized::Witness::Type => Witness::Type,
        sized::Witness::Existential => todo!(),
    }
}

fn closure_name(builder: &Builder) -> String {
    String::from(format!("_closure_{}", builder.functions.len()))
}

fn struct_name(builder: &Builder) -> String {
    String::from(format!("_ClosureEnv_{}", builder.structs.len()))
}

impl fmt::Debug for BlockBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vec = &*self.instrs.borrow();
        write!(f, "{} {:?}", self.name, vec)
    }
}
