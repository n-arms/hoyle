use im::HashSet;
use ir::bridge::{Block, Convention, Expr, Function, Instr, Variable, Witness};
use tree::{typed::Type, String};

use crate::env::Env;

pub fn count_function(env: &mut Env, function: Function) -> Function {
    function
}
/*
    let mut seen = function
        .arguments
        .iter()
        .zip(signature)
        .filter_map(|(arg, convention)| {
            if *convention == Convention::Out {
                Some(arg.name.clone())
            } else {
                None
            }
        })
        .collect();
    let mut body = count_block(env, function.body, &mut seen);
    for arg in function.arguments.iter().rev() {
        count_variable(
            env,
            &Variable {
                name: arg.name.clone(),
                typ: arg.typ.clone(),
            },
            &mut body.instrs,
            &mut seen,
        );
    }
    body.instrs.reverse();
    Function {
        name: function.name,
        arguments: function.arguments,
        body,
    }
}

fn count_block(env: &mut Env, block: Block, seen: &mut HashSet<String>) -> Block {
    let mut instrs = Vec::new();
    for instr in block.instrs.into_iter().rev() {
        match &instr.value {
            Expr::Literal(_) => {}
            Expr::Primitive(_, args) => {
                for arg in args {
                    count_variable(env, &arg, &mut instrs, seen);
                }
            }
            Expr::CallDirect { arguments, .. } => {
                for arg in arguments {
                    if arg.convention != Convention::Out {
                        count_variable(env, &arg.value, &mut instrs, seen);
                    }
                }
            }
            Expr::Move { source, witness } => {
                count_witness(env, &witness, &mut instrs, seen);
                seen.insert(source.name.clone());
            }
            Expr::Copy { source, witness } => {
                count_witness(env, &witness, &mut instrs, seen);
                count_variable(env, &source, &mut instrs, seen);
            }
            Expr::Destroy { witness } => {
                count_witness(env, &witness, &mut instrs, seen);
            }
            Expr::StructPack { arguments, .. } => {
                for arg in arguments {
                    count_variable(env, &arg.value, &mut instrs, seen);
                }
            }
            Expr::If {
                predicate,
                true_branch,
                false_branch,
            } => {
                let (mut true_branch, true_destroyed) = {
                    let mut inner_seen = seen.clone();
                    let block = count_block(env, true_branch.clone(), &mut inner_seen);
                    (block, inner_seen.relative_complement(seen.clone()))
                };
                let (mut false_branch, false_destroyed) = {
                    let mut inner_seen = seen.clone();
                    let block = count_block(env, false_branch.clone(), &mut inner_seen);
                    (block, inner_seen.relative_complement(seen.clone()))
                };
                count_variable(env, predicate, &mut instrs, seen);
                let to_destroy_in_true = false_destroyed
                    .clone()
                    .relative_complement(true_destroyed.clone());
                true_branch
                    .instrs
                    .extend(to_destroy_in_true.into_iter().map(|var| {
                        Instr::new(
                            Variable {
                                name: var.clone(),
                                // TODO: do this correctly
                                typ: Type::integer(),
                            },
                            Expr::Destroy {
                                witness: env.lookup_witness(&var),
                            },
                        )
                    }));
                let to_destroy_in_false = true_destroyed
                    .clone()
                    .relative_complement(false_destroyed.clone());
                false_branch
                    .instrs
                    .extend(to_destroy_in_false.into_iter().map(|var| {
                        Instr::new(
                            Variable {
                                name: var.clone(),
                                // TODO: do this correctly
                                typ: Type::integer(),
                            },
                            Expr::Destroy {
                                witness: env.lookup_witness(&var),
                            },
                        )
                    }));
                true_branch.instrs.reverse();
                false_branch.instrs.reverse();
                seen.extend(true_destroyed);
                seen.extend(false_destroyed);
                count_variable(env, &instr.target, &mut instrs, seen);
                instrs.push(Instr::new(
                    instr.target.clone(),
                    Expr::If {
                        predicate: predicate.clone(),
                        true_branch,
                        false_branch,
                    },
                ));
                continue;
            }
        };
        count_variable(env, &instr.target, &mut instrs, seen);
        instrs.push(instr);
    }
    Block { instrs }
}

fn count_witness(
    env: &Env,
    witness: &Witness,
    instrs: &mut Vec<Instr>,
    seen: &mut HashSet<String>,
) {
    match witness {
        Witness::Trivial { .. } => {}
        Witness::Dynamic { location } => count_variable(env, location, instrs, seen),
        Witness::Type => {}
    }
}

fn count_variable(
    env: &Env,
    variable: &Variable,
    instrs: &mut Vec<Instr>,
    seen: &mut HashSet<String>,
) {
    if !seen.contains(&variable.name) {
        let witness = env.lookup_witness(&variable.name);
        instrs.push(Instr::new(variable.clone(), Expr::Destroy { witness }));
        seen.insert(variable.name.clone());
    }
}
/*
    let (mut body, seen) = count_block(env, function.body);
    let (offsets, seen) = count_block_with(env, function.offsets, seen);
    let (witnesses, seen) = count_block_with(env, function.witnesses, seen);
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
        witnesses,
        offsets,
    }
}

fn count_block(env: &Env, block: Block) -> (Block, HashSet<String>) {
    count_block_with(env, block, HashSet::new())
}
fn count_block_with(
    env: &Env,
    block: Block,
    mut seen: HashSet<String>,
) -> (Block, HashSet<String>) {
    let mut instrs = Vec::new();
    for instr in block.instrs.into_iter().rev() {
        match instr.clone() {
            Instr::Copy {
                target,
                value,
                witness,
            } => {
                if seen.contains(&target.name) || target.name == "_result" {
                    if let Witness::Dynamic { location: witness } = &witness {
                        if !seen.contains(&witness.name) {
                            instrs.push(Instr::Destory {
                                value: witness.clone(),
                                witness: Witness::Trivial { size: 24 },
                            });
                            seen.insert(witness.name.clone());
                        }
                    }
                    if !seen.contains(&value.name) {
                        if !witness.is_trivial() {
                            instrs.push(Instr::Destory {
                                value: value.clone(),
                                witness: witness.clone(),
                            });
                        }
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
                        if !witness.is_trivial() {
                            instrs.push(Instr::Destory {
                                value: arg.clone(),
                                witness,
                            });
                        }
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
*/
*/
