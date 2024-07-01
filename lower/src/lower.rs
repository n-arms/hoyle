use std::rc::Rc;

use crate::env::Env;
use crate::refcount::count_function;
use ir::bridge::{Block, Expr, Function, Instr, Program, Size, Variable};
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
        .map(|arg| env.allocate_variable(arg.name.clone(), arg.typ, arg.size))
        .collect();
    for arg in to_lower.arguments.iter().rev() {
        if arg.typ != Type::typ() {
            let witness = expr(&mut env, &arg.witness, &mut instrs);
            env.set_witness(arg.name.clone(), witness)
        };
    }
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
        sized::Expr::Variable { name, .. } => env.lookup_variable(&name.name),
        sized::Expr::Literal { literal } => {
            let name = env.fresh_name();
            let result = env.allocate_variable(name, Type::float(), sized::Size::new_static(8));
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
            if let Some(witness) = tag.witness.as_ref() {
                let lowered_witness = expr(env, witness, instrs);
                env.set_witness(name.clone(), lowered_witness);
            }
            let result = env.allocate_variable(name, tag.result.clone(), tag.size.clone());
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
                let lowered_value = expr(env, value, instrs);
                let variable =
                    env.allocate_variable(name.name.clone(), typ.clone(), name.size.clone());
                let witness = expr(env, &name.witness, instrs);
                env.set_witness(name.name.clone(), witness.clone());
                instrs.push(Instr::Copy {
                    target: variable,
                    value: lowered_value,
                    witness: Some(witness),
                });
            }
        }
    }
    expr(env, &to_lower.result, instrs)
}
