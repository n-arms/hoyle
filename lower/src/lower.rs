use core::fmt;
use std::cell::RefCell;

use crate::env::{Env, GlobalEnv};
use crate::refcount::count_function;
use ir::bridge::{
    Block, CallArgument, Convention, Expr, Function, Instr, Program, Variable, Witness,
};
use tree::sized::{self};
use tree::typed::Type;
use tree::String;

pub fn program(to_lower: &sized::Program) -> (Program, Vec<Env>) {
    let mut global = GlobalEnv::default();
    for func in &to_lower.functions {
        let num_args = func.arguments.len() - 1;
        let mut convention = vec![Convention::Out];
        convention.extend(vec![Convention::In; num_args]);
        global.define_function(func.name.clone(), convention);
    }
    global.define_function(String::from("F64"), vec![Convention::Out]);
    let (functions, envs) = to_lower
        .functions
        .iter()
        .map(|func| function(global.clone(), func))
        .unzip();
    let program = Program {
        structs: to_lower.structs.clone(),
        functions,
    };
    (program, envs)
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

fn variable(
    env: &mut Env,
    to_lower: &sized::Witness,
    name: String,
    typ: Type,
    instrs: &BlockBuilder,
) -> Variable {
    if let Some(var) = env.try_define_variable(name.clone(), typ.clone()) {
        var
    } else {
        let witness = witness(env, to_lower, instrs);
        env.define_variable(name, typ, witness)
    }
}

fn function(global: GlobalEnv, to_lower: &sized::Function) -> (Function, Env) {
    let mut env = Env::new(global);
    let body_builder = BlockBuilder::new("body");
    let lowered_arguments: Vec<_> = to_lower
        .arguments
        .iter()
        .cloned()
        .map(|arg| variable(&mut env, &arg.witness, arg.name, arg.typ, &body_builder))
        .collect();
    let result = expr(&mut env, &to_lower.body, &body_builder);
    body_builder.push(Instr::new(
        Variable {
            name: lowered_arguments[0].name.clone(),
            typ: lowered_arguments[0].typ.clone(),
        },
        Expr::Move {
            source: result.clone(),
            witness: env.lookup_witness(&result.name),
        },
    ));
    let func = count_function(
        &mut env,
        Function {
            name: to_lower.name.clone(),
            arguments: lowered_arguments,
            body: body_builder.build(),
        },
    );
    (func, env)
}

pub fn expr(env: &mut Env, to_lower: &sized::Expr, instrs: &BlockBuilder) -> Variable {
    match to_lower {
        sized::Expr::Variable { name, typ } => Variable {
            name: name.name.clone(),
            typ: typ.clone(),
        },
        sized::Expr::Literal { literal } => {
            let name = env.fresh_name();
            let var = env.define_variable(name, Type::float(), Witness::Trivial { size: 8 });
            instrs.push(Instr::new(var.clone(), Expr::Literal(literal.clone())));
            var
        }
        sized::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let name = env.fresh_name();
            let result = variable(env, &tag.witness, name, tag.result.clone(), instrs);
            let mut lowered_arguments: Vec<_> = arguments
                .iter()
                .map(|to_lower| expr(env, to_lower, instrs))
                .collect();
            lowered_arguments.insert(0, result.clone());
            let signature: Vec<_> = env.lookup_convention(function).iter().copied().collect();
            let tagged_arguments: Vec<_> = lowered_arguments
                .into_iter()
                .zip(signature)
                .map(|(value, convention)| CallArgument { value, convention })
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
    }
}

fn block(env: &mut Env, to_lower: &sized::Block, instrs: &BlockBuilder) -> Variable {
    for stmt in &to_lower.stmts {
        match stmt {
            sized::Statement::Let { name, typ, value } => {
                let witness = witness(env, &name.witness, instrs);
                let source = expr(env, value, instrs);
                let variable = env.define_variable(name.name.clone(), typ.clone(), witness.clone());
                let value = Expr::Copy { source, witness };
                instrs.push(Instr::new(variable, value));
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
    }
}

impl fmt::Debug for BlockBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vec = &*self.instrs.borrow();
        write!(f, "{} {:?}", self.name, vec)
    }
}
