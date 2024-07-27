use core::fmt;
use std::cell::RefCell;

use crate::env::Env;
use crate::refcount::count_function;
use ir::bridge::{Atom, Block, Expr, Function, Instr, Program, Variable, Witness};
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

    fn build(self, result: Atom) -> Block {
        Block {
            instrs: self.instrs.into_inner(),
            result,
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

fn function(to_lower: &sized::Function) -> Function {
    let mut env = Env::new();
    let body_builder = BlockBuilder::new("body");
    let lowered_arguments: Vec<_> = to_lower
        .arguments
        .iter()
        .cloned()
        .map(|arg| variable(&mut env, &arg.witness, arg.name, arg.typ, &body_builder))
        .collect();
    let result = expr(&mut env, &to_lower.body, &body_builder);
    let mut func = count_function(
        &mut env,
        Function {
            name: to_lower.name.clone(),
            arguments: lowered_arguments,
            body: body_builder.build(result),
        },
    );
    let offsets = crate::offset::function(&mut env, &mut func);
    println!("{:?}", offsets);
    func
}

pub fn expr(env: &mut Env, to_lower: &sized::Expr, instrs: &BlockBuilder) -> Atom {
    match to_lower {
        sized::Expr::Variable { name, typ } => Atom::Variable(Variable {
            name: name.name.clone(),
            typ: typ.clone(),
        }),
        sized::Expr::Literal { literal } => Atom::Literal(literal.clone()),
        sized::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let mut lowered_arguments: Vec<_> = arguments
                .iter()
                .map(|to_lower| expr(env, to_lower, instrs))
                .collect();
            let name = env.fresh_name();
            let result = variable(env, &tag.witness, name, tag.result, instrs);

            instrs.push(Instr::new(
                result,
                Expr::CallDirect {
                    function: function.clone(),
                    arguments: lowered_arguments,
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
                let lowered_value = expr(env, value, instrs);
                let variable = env.define_variable(name.name.clone(), typ.clone(), witness.clone());
                instrs.push(Instr::Copy {
                    target: variable,
                    value: lowered_value,
                    witness,
                });
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
