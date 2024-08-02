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
        function(to_lower, &mut builder);
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

fn function<'a>(to_lower: &sized::Function, builder: &'a mut Builder) -> &'a mut Function {
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
    builder.functions.push(count_function(Function {
        name: to_lower.name.clone(),
        arguments: lowered_arguments,
        body: body_builder.build(),
        names,
    }));
    builder.functions.last_mut().unwrap()
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
            let get_value = |name| String::from(format!("{env_name}_get_{name}"));
            let get_generic = |id| String::from(format!("_witness_get_{id}"));
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
                arguments.extend([
                    sized::Argument {
                        name: String::from("_env"),
                        typ: env_type,
                        witness: sized::Witness::Dynamic {
                            value: Box::new(sized::Expr::Variable {
                                name: sized::Variable {
                                    name: String::from("_env_witness"),
                                    witness: sized::Witness::Type,
                                },
                                typ: Type::typ(),
                            }),
                        },
                    },
                    sized::Argument {
                        name: String::from("_env_witness"),
                        typ: Type::typ(),
                        witness: sized::Witness::Type,
                    },
                ]);
                arguments.extend(tag.value_captures.iter().cloned());
                arguments.extend(tag.type_captures.iter().cloned());
                sized::Function {
                    name: function_name,
                    generics: Vec::new(),
                    arguments,
                    result: body.get_type(),
                    body: body.as_ref().clone(),
                }
            };
            let lowered_func = function(&mocked_function, builder);
            let fake_arguments = tag.value_captures.len() + tag.type_captures.len();
            let real_arguments = lowered_func.arguments.len() - fake_arguments;
            let env_witness_variable =
                Variable::new(String::from("_env_witness"), Type::typ(), Witness::Type);
            let env_variable = Variable::new(
                String::from("_env"),
                env_type,
                Witness::Dynamic {
                    location: env_witness_variable,
                },
            );
            let type_preamble = tag.type_captures.iter().enumerate().map(|(i, arg)| {
                let result = Variable::new(arg.name.clone(), arg.typ.clone(), Witness::Type);
                let result_arg = CallArgument {
                    value: Value::Copy(result),
                    convention: Convention::Out,
                };
                let env_arg = CallArgument {
                    value: Value::Copy(env_witness_variable.clone()),
                    convention: Convention::In,
                };
                let arguments = vec![result_arg, env_arg];
                Instr::new(
                    result.clone(),
                    Expr::CallDirect {
                        function: get_generic(i),
                        arguments,
                    },
                )
            });
            let witness_preamble = BlockBuilder::new("witness preamble");
            let value_preamble = tag.value_captures.iter().map(|arg| {
                let witness = witness(env, &arg.witness, &witness_preamble, builder);
                let target = Variable::new(arg.name.clone(), arg.typ.clone(), witness);
                Instr::new(
                    target,
                    Expr::Unpack {
                        value: env_variable.clone(),
                        field: arg.name.clone(),
                    },
                )
            });
            let preamble = type_preamble
                .chain(witness_preamble.build().instrs)
                .chain(value_preamble)
                .collect();
            let instrs = lowered_func.body.instrs;
            lowered_func.body.instrs = preamble;
            lowered_func.body.instrs.extend(instrs);
            todo!()
        }
    }
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
