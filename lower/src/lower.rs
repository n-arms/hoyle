use std::rc::Rc;

use crate::env::Env;
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
        .map(|arg| env.allocate_variable(arg.name.clone(), arg.typ, arg.size, Some(arg.witness)))
        .collect();
    let result = expr(&mut env, &to_lower.body, &mut instrs);
    let variable = lowered_arguments[0].clone();
    let witness_expr = env.lookup_witness(&variable.name);
    let witness = witness_expr.map(|e| expr(&mut env, &e, &mut instrs));
    instrs.push(Instr::Copy {
        target: variable,
        value: result,
        witness,
    });
    let body = Block { instrs };
    Function {
        name: to_lower.name.clone(),
        arguments: lowered_arguments,
        body,
    }
}

fn expr(env: &mut Env, to_lower: &sized::Expr, instrs: &mut Vec<Instr>) -> Variable {
    match to_lower {
        sized::Expr::Variable { name, .. } => env.lookup_variable(&name.name),
        sized::Expr::Literal { literal } => {
            let name = env.fresh_name();
            let result =
                env.allocate_variable(name, Type::float(), sized::Size::new_static(8), None);
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
            let witness = tag.witness.as_ref().map(AsRef::as_ref).map(Clone::clone);
            let result = env.allocate_variable(name, tag.result.clone(), tag.size.clone(), witness);
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
                let variable = env.allocate_variable(
                    name.name.clone(),
                    typ.clone(),
                    name.size.clone(),
                    Some(*name.witness.clone()),
                );
                let witness = expr(env, &name.witness, instrs);
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
