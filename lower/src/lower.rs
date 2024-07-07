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

fn function(to_lower: &sized::Function) -> Function {
    let mut env = Env::new();
    let mut instrs = Vec::new();
    let lowered_arguments: Vec<_> = to_lower
        .arguments
        .iter()
        .cloned()
        .map(|arg| {
            let witness = witness(&mut env, &arg.witness, &mut instrs);
            env.define_variable(arg.name, arg.typ, witness)
        })
        .collect();
    let result = expr(&mut env, &to_lower.body, &mut instrs);
    let variable = lowered_arguments[0].clone();
    let witness = env.lookup_witness(&variable.name);
    instrs.push(Instr::Copy {
        target: variable,
        value: result,
        witness,
    });
    let body = Block { instrs };
    count_function(
        &mut env,
        Function {
            name: to_lower.name.clone(),
            arguments: lowered_arguments,
            body,
        },
    )
}

pub fn expr(env: &mut Env, to_lower: &sized::Expr, instrs: &mut Vec<Instr>) -> Variable {
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
                .map(|to_lower| expr(env, to_lower, instrs))
                .collect();
            let name = env.fresh_name();
            let witness = witness(env, &tag.witness, instrs);
            let result = env.define_variable(name, tag.result.clone(), witness);
            lowered_arguments.insert(0, result.clone());

            instrs.push(Instr::CallDirect {
                function: function.clone(),
                arguments: lowered_arguments,
            });

            result
        }
        sized::Expr::Block(to_lower) => block(env, to_lower, instrs),
    }
}

fn block(env: &mut Env, to_lower: &sized::Block, instrs: &mut Vec<Instr>) -> Variable {
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

fn witness(env: &mut Env, to_lower: &sized::Witness, instrs: &mut Vec<Instr>) -> Witness {
    match to_lower {
        sized::Witness::Trivial { size } => Witness::Trivial { size: *size },
        sized::Witness::Dynamic { value } => Witness::Dynamic {
            location: expr(env, value.as_ref(), instrs),
        },
    }
}
