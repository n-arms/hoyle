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
        .map(|arg| {
            let size = lower_size(&mut env, &arg.size, &mut instrs);
            let witness = expr(&mut env, &arg.witness, &mut instrs);
            env.allocate_variable(arg.name, arg.typ, size, Some(witness))
        })
        .collect();
    let result = expr(&mut env, &to_lower.body, &mut instrs);
    instrs.push(Instr::Copy {
        target: lowered_arguments[0].clone(),
        value: result,
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
            let result = env.allocate_variable(name, Type::float(), Size::new_static(8), None);
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
            let lowered_arguments: Vec<_> = arguments
                .iter()
                .map(|to_lower| expr(env, to_lower, instrs))
                .collect();
            let name = env.fresh_name();
            let witness = tag.witness.clone().map(|e| expr(env, e.as_ref(), instrs));
            let size = lower_size(env, &tag.size, instrs);
            let result = env.allocate_variable(name, tag.result.clone(), size, witness);
            let mut offset = result.offset.clone() + result.size.clone();
            let argument_offsets = lowered_arguments.iter().map(|arg| {
                let slot = offset.clone();
                offset += arg.size.clone();
                slot
            });
            for (i, (arg, offset)) in lowered_arguments.iter().zip(argument_offsets).enumerate() {
                instrs.push(Instr::Copy {
                    value: arg.clone(),
                    target: Variable {
                        name: String::from(format!("_{}", i)),
                        typ: arg.typ.clone(),
                        size: arg.size.clone(),
                        offset,
                        witness: arg.witness.clone(),
                    },
                });
            }
            instrs.push(Instr::CallDirect {
                function: function.clone(),
            });

            result
        }
        sized::Expr::Block(to_lower) => block(env, to_lower, instrs),
    }
}

fn lower_size(env: &mut Env, size: &sized::Size, instrs: &mut Vec<Instr>) -> Size {
    let dynamic = size
        .dynamic
        .iter()
        .map(|to_lower| expr(env, to_lower, instrs))
        .collect();
    Size {
        static_size: size.static_size,
        dynamic,
    }
}

fn block(env: &mut Env, to_lower: &sized::Block, instrs: &mut Vec<Instr>) -> Variable {
    for stmt in &to_lower.stmts {
        match stmt {
            sized::Statement::Let { name, typ, value } => {
                let lowered_value = expr(env, value, instrs);
                let size = lower_size(env, &name.size, instrs);
                let witness = expr(env, name.witness.as_ref(), instrs);
                let variable =
                    env.allocate_variable(name.name.clone(), typ.clone(), size, Some(witness));
                instrs.push(Instr::Copy {
                    target: variable,
                    value: lowered_value,
                });
            }
        }
    }
    expr(env, &to_lower.result, instrs)
}
