use im::HashSet;
use ir::bridge::{Block, Function, Instr};
use tree::String;

use crate::env::Env;
use crate::lower::expr;

pub fn count_function(env: &mut Env, function: Function) -> Function {
    let (mut body, seen) = count_block(env, function.body);
    for arg in &function.arguments {
        if !seen.contains(&arg.name) && arg.name != "_result" {
            let witness = env.lookup_witness(&arg.name);

            body.instrs.push(Instr::Destory {
                value: arg.clone(),
                witness,
            });
        }
    }
    Function {
        name: function.name,
        arguments: function.arguments,
        body,
    }
}

fn count_block(env: &Env, block: Block) -> (Block, HashSet<String>) {
    let mut seen = HashSet::new();
    let mut instrs = Vec::new();
    for instr in block.instrs.into_iter().rev() {
        match instr.clone() {
            Instr::Copy {
                target,
                value,
                witness,
            } => {
                if seen.contains(&target.name) || target.name == "_result" {
                    if let Some(witness) = witness.as_ref() {
                        if !seen.contains(&witness.name) {
                            instrs.push(Instr::Destory {
                                value: witness.clone(),
                                witness: None,
                            });
                            seen.insert(witness.name.clone());
                        }
                    }
                    if !seen.contains(&value.name) {
                        instrs.push(Instr::Destory {
                            value: value.clone(),
                            witness: witness.clone(),
                        });
                        seen.insert(value.name.clone());
                    }
                    instrs.push(Instr::Copy {
                        target,
                        value,
                        witness,
                    });
                }
            }
            Instr::Set { target, .. } => {
                if seen.contains(&target.name) {
                    instrs.push(instr);
                }
            }
            Instr::CallDirect {
                function,
                arguments,
            } => {
                for arg in &arguments {
                    if !seen.contains(&arg.name) {
                        let witness = env.lookup_witness(&arg.name);
                        instrs.push(Instr::Destory {
                            value: arg.clone(),
                            witness,
                        });
                        seen.insert(arg.name.clone());
                    }
                }
                instrs.push(Instr::CallDirect {
                    function: function.clone(),
                    arguments,
                });
            }
            _ => {
                instrs.push(instr);
            }
        }
    }
    instrs.reverse();
    (Block { instrs }, seen)
}
