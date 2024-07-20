use core::fmt;
use std::cell::RefCell;

use crate::env::Env;
use crate::refcount::count_function;
use ir::bridge::{Block, Expr, Function, Instr, Program, Variable, Witness};
use tree::sized::{self};
use tree::typed::Type;
use tree::String;

pub fn program(to_lower: &sized::Program) -> Program {
    Program {
        structs: to_lower.structs.clone(),
        functions: to_lower.functions.iter().map(function).collect(),
    }
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
        println!("adding instr {} to {}", instr, self.name);
        self.instrs.borrow_mut().push(instr);
    }

    fn build(self) -> Block {
        Block {
            instrs: self.instrs.into_inner(),
        }
    }
}

fn function(to_lower: &sized::Function) -> Function {
    let mut env = Env::new();
    let body_builder = BlockBuilder::new("body");
    let witnesses_builder = BlockBuilder::new("witnesses");
    let lowered_arguments: Vec<_> = to_lower
        .arguments
        .iter()
        .cloned()
        .map(|arg| {
            let witness = witness(&mut env, &arg.witness, &witnesses_builder);
            env.define_variable(arg.name, arg.typ, witness)
        })
        .collect();
    println!("{witnesses_builder:?}");
    let result = expr(&mut env, &to_lower.body, &body_builder, &witnesses_builder);
    let variable = lowered_arguments[0].clone();
    let witness = env.lookup_witness(&variable.name);
    body_builder.push(Instr::Copy {
        target: variable,
        value: result,
        witness,
    });
    let mut func = count_function(
        &mut env,
        Function {
            name: to_lower.name.clone(),
            arguments: lowered_arguments,
            body: body_builder.build(),
            witnesses: witnesses_builder.build(),
            offsets: Block { instrs: Vec::new() },
        },
    );
    let offsets = crate::offset::function(&mut env, &mut func);
    println!("{:?}", offsets);
    func
}

pub fn expr(
    env: &mut Env,
    to_lower: &sized::Expr,
    instrs: &BlockBuilder,
    witnesses: &BlockBuilder,
) -> Variable {
    match to_lower {
        sized::Expr::Variable { name, typ } => Variable {
            name: name.name.clone(),
            typ: typ.clone(),
        },
        sized::Expr::Literal { literal } => {
            let name = env.fresh_name();
            let result = env.define_variable(name, Type::float(), Witness::trivial(8));
            instrs.push(Instr::Set {
                target: result.clone(),
                expr: Expr::Literal(literal.clone()),
            });
            result
        }
        sized::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let mut lowered_arguments: Vec<_> = arguments
                .iter()
                .map(|to_lower| expr(env, to_lower, instrs, witnesses))
                .collect();
            let name = env.fresh_name();
            let witness = witness(env, &tag.witness, witnesses);
            let result = env.define_variable(name, tag.result.clone(), witness);
            lowered_arguments.insert(0, result.clone());

            instrs.push(Instr::CallDirect {
                function: function.clone(),
                arguments: lowered_arguments,
            });

            result
        }
        sized::Expr::Block(to_lower) => block(env, to_lower, instrs, witnesses),
    }
}

fn block(
    env: &mut Env,
    to_lower: &sized::Block,
    instrs: &BlockBuilder,
    witnesses: &BlockBuilder,
) -> Variable {
    for stmt in &to_lower.stmts {
        match stmt {
            sized::Statement::Let { name, typ, value } => {
                let witness = witness(env, &name.witness, witnesses);
                let lowered_value = expr(env, value, instrs, witnesses);
                let variable = env.define_variable(name.name.clone(), typ.clone(), witness.clone());
                instrs.push(Instr::Copy {
                    target: variable,
                    value: lowered_value,
                    witness,
                });
            }
        }
    }
    expr(env, &to_lower.result, instrs, witnesses)
}

fn witness(env: &mut Env, to_lower: &sized::Witness, witnesses: &BlockBuilder) -> Witness {
    match to_lower {
        sized::Witness::Trivial { size } => Witness::Trivial { size: *size },
        sized::Witness::Dynamic { value } => Witness::Dynamic {
            location: expr(env, value.as_ref(), witnesses, witnesses),
        },
    }
}

impl fmt::Debug for BlockBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vec = &*self.instrs.borrow();
        write!(f, "{} {:?}", self.name, vec)
    }
}
