use core::fmt;
use std::cell::RefCell;

use crate::env::Env;
use crate::refcount::count_function;
use ir::bridge::{
    Block, BuilderArgument, CallArgument, Convention, Expr, Function, Instr, PackField, Program,
    Struct, StructBuilder, Value, Variable, Witness,
};
use tree::sized::{self};
use tree::typed::Type;
use tree::String;

pub fn program(to_lower: &sized::Program) -> Program {
    let structs = to_lower
        .structs
        .iter()
        .map(|to_lower| strukt(to_lower))
        .collect();
    let functions = to_lower
        .functions
        .iter()
        .map(|func| function(func))
        .collect();
    Program { structs, functions }
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

fn strukt(to_lower: &sized::Struct) -> Struct {
    let mut env = Env::new();
    let instrs = BlockBuilder::new("struct");
    let fields = to_lower
        .tag
        .fields
        .iter()
        .map(|field| expr(&mut env, field, &instrs))
        .collect();

    let block = instrs.build();
    let lowered_builder = StructBuilder {
        arguments: vec![BuilderArgument {
            name: Variable {
                name: String::from("_result"),
                typ: Type::typ(),
            },
            convention: Convention::Out,
        }],
        block,
        fields,
        names: env.name_source,
    };

    Struct {
        definition: to_lower.clone(),
        builder: lowered_builder,
    }
}

fn function(to_lower: &sized::Function) -> Function {
    let mut env = Env::new();
    let body_builder = BlockBuilder::new("body");
    let lowered_arguments: Vec<_> = to_lower
        .arguments
        .iter()
        .cloned()
        .map(|arg| env.define_variable(arg.name, arg.typ))
        .collect();
    let result = expr(&mut env, &to_lower.body, &body_builder);
    let result_witness = witness(&mut env, &to_lower.body.get_witness(), &body_builder);
    body_builder.push(Instr::new(
        Variable {
            name: lowered_arguments[0].name.clone(),
            typ: lowered_arguments[0].typ.clone(),
        },
        Expr::mov(result.clone(), result_witness),
    ));
    let names = env.name_source.clone();
    count_function(
        &mut env,
        Function {
            name: to_lower.name.clone(),
            arguments: lowered_arguments,
            body: body_builder.build(),
            names,
        },
    )
}

pub fn expr(env: &mut Env, to_lower: &sized::Expr, instrs: &BlockBuilder) -> Variable {
    match to_lower {
        sized::Expr::Variable { name, typ } => Variable {
            name: name.name.clone(),
            typ: typ.clone(),
        },
        sized::Expr::Literal { literal } => {
            let name = env.fresh_name();
            let var = env.define_variable(name, literal.get_type());
            instrs.push(Instr::new(var.clone(), Expr::Literal(literal.clone())));
            var
        }
        sized::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let name = env.fresh_name();
            let result = env.define_variable(name, tag.result.clone());
            let mut lowered_arguments: Vec<_> = arguments
                .iter()
                .map(|to_lower| (expr(env, to_lower, instrs), to_lower.get_witness()))
                .collect();
            lowered_arguments.insert(0, (result.clone(), tag.witness.clone()));
            let tagged_arguments: Vec<_> = lowered_arguments
                .into_iter()
                .zip(tag.signature.clone())
                .map(|((value, witness), convention)| CallArgument {
                    value: copy_variable(env, value, &witness, instrs),
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
        sized::Expr::Block(to_lower) => block(env, to_lower, instrs),
        sized::Expr::Primitive {
            primitive,
            arguments,
        } => {
            let name = env.fresh_name();
            let result = env.define_variable(name, arguments[0].get_type());
            let lowered_args = arguments
                .iter()
                .map(|to_lower| expr(env, to_lower, instrs))
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
            let result = env.define_variable(name, tag.result.clone());
            let lowered_fields = fields
                .iter()
                .map(|field| {
                    let value = expr(env, &field.value, instrs);
                    PackField {
                        name: field.name.clone(),
                        value: copy_variable(env, value, &field.value.get_witness(), instrs),
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
            let witness = witness(env, &tag.witness, instrs);
            let name = env.fresh_name();
            let result = env.define_variable(name, true_branch.get_type());
            let lowered_predicate = expr(env, &predicate, instrs);
            let true_instrs = BlockBuilder::new("true branch");
            let lowered_true = expr(env, &true_branch, &true_instrs);
            true_instrs.push(Instr::new(
                result.clone(),
                Expr::copy(lowered_true, witness.clone()),
            ));
            let false_instrs = BlockBuilder::new("false branch");
            let lowered_false = expr(env, &false_branch, &false_instrs);
            false_instrs.push(Instr::new(
                result.clone(),
                Expr::copy(lowered_false, witness.clone()),
            ));
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
    }
}

fn copy_variable(
    env: &mut Env,
    value: Variable,
    wit: &sized::Witness,
    instrs: &BlockBuilder,
) -> Value {
    let witness = witness(env, wit, instrs);
    Value::Copy { value, witness }
}

fn block(env: &mut Env, to_lower: &sized::Block, instrs: &BlockBuilder) -> Variable {
    for stmt in &to_lower.stmts {
        match stmt {
            sized::Statement::Let { name, typ, value } => {
                let target = env.define_variable(name.name.clone(), typ.clone());
                let source = expr(env, value, instrs);
                let value = copy_variable(env, source, &name.witness, instrs);
                instrs.push(Instr::new(target, Expr::Value(value)));
            }
        }
    }
    expr(env, &to_lower.result, instrs)
}

fn witness(env: &mut Env, to_lower: &sized::Witness, instrs: &BlockBuilder) -> Witness {
    match to_lower {
        sized::Witness::Trivial { size } => Witness::Trivial { size: *size },
        sized::Witness::Dynamic { value } => Witness::Dynamic {
            location: expr(env, value.as_ref(), instrs),
        },
        sized::Witness::Type => Witness::Type,
    }
}

impl fmt::Debug for BlockBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vec = &*self.instrs.borrow();
        write!(f, "{} {:?}", self.name, vec)
    }
}
