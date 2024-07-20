use core::fmt;

use im::{HashMap, HashSet};
use ir::bridge::{Block, Expr, Function, Instr, Variable, Witness};
use tree::{
    sized::Primitive,
    typed::{Literal, Type},
    String,
};

use crate::env::Env;

#[derive(Clone)]
pub enum Offset {
    Trivial(i64),
    Dynamic(Variable),
}

#[derive(Debug)]
pub struct Offsets {
    pub offsets: HashMap<String, Offset>,
}

/*
fn function(env: &mut Env, to_offset: &mut Function) -> Offsets {
    let mut offset = Offset::Trivial(0);
    let mut offsets = HashMap::new();
    let mut prefix = Vec::new();
    for arg in to_offset.arguments.iter().rev() {
        offsets.insert(arg.name.clone(), offset);
        offset = match (env.lookup_witness(&arg.name), offset) {
            // offset contains the start of the variable
            (Witness::Trivial { size }, Offset::Trivial(offset)) => {
                Offset::Trivial(size as i64 + offset)
            }
            (Witness::Dynamic { location }, Offset::Trivial(static_edge)) => {
                let static_edge_name = env.fresh_name();
                let static_edge_var =
                    env.define_variable(static_edge_name, Type::integer(), Witness::trivial(8));
                prefix.push(Instr::Set {
                    target: static_edge_var,
                    expr: Expr::Literal(Literal::Integer(static_edge + 16)),
                });
                let new_edge = env.fresh_name();
                let new_edge_var =
                    env.define_variable(new_edge, Type::integer(), Witness::trivial(8));
                prefix.push(Instr::Set {
                    target: new_edge_var,
                    expr: Expr::Primitive(Primitive::Add, vec![static_edge_var, location]),
                });
                Offset::Dynamic(new_edge_var)
            }
            (Witness::Trivial { size }, Offset::Dynamic(location)) => {
                let new_edge_name = env.fresh_name();
                let new_edge_var =
                    env.define_variable(new_edge_name, Type::integer(), Witness::trivial(8));
            }
            (Witness::Dynamic { location }, Offset::Dynamic(_)) => todo!(),
        }
    }
    offsets
}

fn block(
    offsets: &mut HashMap<String, Offset>,
    offset: i64,
    env: &mut Env,
    to_offset: Block,
) -> Block {
    let variables = to_offset.instrs.iter().flat_map(instr_variables);
    let mut static_vars = HashMap::new();
    let mut dynamic_vars = HashMap::new();

    for var in variables {
        match env.lookup_witness(var) {
            Witness::Trivial { size } => {
                static_vars.insert(var, size);
            }
            Witness::Dynamic { location } => {
                dynamic_vars.insert(var.clone(), location);
            }
        }
    }

    let mut static_offset = offset;

    for (var, size) in static_vars {
        offsets.insert(var.clone(), Offset::Trivial(static_offset));
        static_offset += size as i64;
    }

    let mut dynamic_offset = None;

    let mut instrs = Vec::new();

    for instr in to_offset.instrs {
        for var in instr_variables(&instr) {
            if offsets.contains_key(var) {
                continue;
            }
            if let Some(witness) = dynamic_vars.get(var) {
                let current = match dynamic_offset {
                    Some(current) => current,
                    None => {
                        let static_edge_name = env.fresh_name();
                        let var = env.define_variable(
                            static_edge_name,
                            Type::integer(),
                            Witness::trivial(8),
                        );
                        // final 8 bytes to store the size of the static offset to use in dynamic calculations
                        static_offset += 8;
                        instrs.push(Instr::Set {
                            target: var.clone(),
                            expr: Expr::Literal(Literal::Integer(static_offset as i64)),
                        });
                        var
                    }
                };
                offsets.insert(var.clone(), Offset::Dynamic(current));
                let offset_name = env.fresh_name();
                let offset_var =
                    env.define_variable(offset_name, Type::integer(), Witness::trivial(8));
                let size = unimplemented!("Extract the size out of the witness table: needs tuple indexing to be implemented");
                instrs.push(Instr::Set {
                    target: offset_var.clone(),
                    expr: Expr::Primitive(Primitive::Add, vec![current, size]),
                });
                dynamic_offset = Some(offset_var);
            }
        }
        instrs.push(instr);
    }

    Block { instrs }
}
*/
/*
pub(crate) fn function(env: &mut Env, to_offset: &mut Function) -> Offsets {
    let mut arg_offset = Offset::Trivial(0);
    let mut prefix_offset = 0;
    let mut arg_prefix = Vec::new();
    let mut offsets = HashMap::new();
    for name in arg_variables(&to_offset.arguments) {
        println!(
            "on name {} with prefix_offset {} and arg_offset {:?}",
            name, prefix_offset, arg_offset
        );
        arg_offset = match (arg_offset, env.lookup_witness(name)) {
            (Offset::Trivial(edge), Witness::Trivial { size }) => {
                offsets.insert(name.clone(), Offset::Trivial(edge));
                Offset::Trivial(edge - size as i64)
            }
            (Offset::Trivial(edge), Witness::Dynamic { location }) => {
                let edge_name = env.fresh_name();
                let edge_var =
                    env.define_variable(edge_name.clone(), Type::integer(), Witness::trivial(8));
                let new_name = env.fresh_name();
                let new_var =
                    env.define_variable(new_name.clone(), Type::integer(), Witness::trivial(8));
                offsets.insert(edge_name.clone(), Offset::Trivial(prefix_offset));
                offsets.insert(new_name, Offset::Trivial(prefix_offset + 8));
                prefix_offset += 16;
                arg_prefix.push(Instr::Set {
                    target: edge_var.clone(),
                    expr: Expr::Literal(Literal::Integer(edge)),
                });
                arg_prefix.push(Instr::Set {
                    target: new_var.clone(),
                    expr: Expr::Primitive(Primitive::Sub, vec![edge_var, location]),
                });
                offsets.insert(name.clone(), Offset::Trivial(edge));
                Offset::Dynamic(new_var)
            }
            (Offset::Dynamic(location), Witness::Trivial { size }) => {
                let size_name = env.fresh_name();
                let size_var =
                    env.define_variable(size_name.clone(), Type::integer(), Witness::trivial(8));
                offsets.insert(size_name.clone(), Offset::Trivial(prefix_offset));
                let new_name = env.fresh_name();
                let new_var =
                    env.define_variable(new_name.clone(), Type::integer(), Witness::trivial(8));
                offsets.insert(new_name.clone(), Offset::Trivial(prefix_offset + 8));
                prefix_offset += 16;
                arg_prefix.push(Instr::Set {
                    target: size_var.clone(),
                    expr: Expr::Literal(Literal::Integer(size as i64)),
                });
                arg_prefix.push(Instr::Set {
                    target: new_var.clone(),
                    expr: Expr::Primitive(Primitive::Sub, vec![location.clone(), new_var.clone()]),
                });
                offsets.insert(name.clone(), Offset::Dynamic(location));
                Offset::Dynamic(new_var)
            }
            (Offset::Dynamic(offset_loc), Witness::Dynamic { location }) => {
                let new_name = env.fresh_name();
                let new_var =
                    env.define_variable(new_name.clone(), Type::integer(), Witness::trivial(8));
                offsets.insert(new_name.clone(), Offset::Trivial(prefix_offset));
                prefix_offset += 8;
                arg_prefix.push(Instr::Set {
                    target: new_var.clone(),
                    expr: Expr::Primitive(Primitive::Sub, vec![offset_loc.clone(), location]),
                });
                offsets.insert(name.clone(), Offset::Dynamic(offset_loc));
                Offset::Dynamic(new_var)
            }
        };
    }
    for (i, instr) in arg_prefix.into_iter().enumerate() {
        to_offset.body.instrs.insert(i, instr);
    }
    Offsets { offsets }
}
*/
pub(crate) fn function(env: &mut Env, to_offset: &mut Function) -> Offsets {
    let mut static_edge = 0;
    let mut dynamic_edge = {
        let name = env.fresh_name();
        env.define_variable(name, Type::integer(), Witness::trivial(8))
    };
    let static_edge_var = dynamic_edge.clone();
    let mut offsets = HashMap::new();

    {
        let mut static_arg_edge = 0;
        let mut dynamic_arg_edge = {
            let name = env.fresh_name();
            env.define_variable(name, Type::integer(), Witness::trivial(8))
        };
        static_edge += 8;
        let static_arg_var = dynamic_arg_edge.clone();
        for arg in arg_variables(&to_offset.arguments) {
            match env.lookup_witness(arg) {
                Witness::Trivial { size } => {
                    static_arg_edge -= size as i64;
                    offsets.insert(arg.clone(), Offset::Trivial(static_arg_edge));
                }
                Witness::Dynamic { location } => {
                    let new_dynamic_edge = {
                        let name = env.fresh_name();
                        env.define_variable(name, Type::integer(), Witness::trivial(8))
                    };
                    to_offset.offsets.instrs.push(Instr::Set {
                        target: new_dynamic_edge.clone(),
                        expr: Expr::Primitive(Primitive::Sub, vec![dynamic_arg_edge, location]),
                    });
                    offsets.insert(arg.clone(), Offset::Dynamic(new_dynamic_edge.clone()));
                    offsets.insert(new_dynamic_edge.name.clone(), Offset::Trivial(static_edge));
                    dynamic_arg_edge = new_dynamic_edge;
                    static_edge += 8;
                }
            }
        }
        to_offset.offsets.instrs.insert(
            0,
            Instr::Set {
                target: static_arg_var,
                expr: Expr::Literal(Literal::Integer(static_arg_edge)),
            },
        )
    }

    {
        for var in body_variables(&to_offset.witnesses)
            .into_iter()
            .chain(body_variables(&to_offset.body))
        {
            if offsets.contains_key(var) {
                continue;
            }
            match env.lookup_witness(var) {
                Witness::Trivial { size } => {
                    offsets.insert(var.clone(), Offset::Trivial(static_edge));
                    static_edge += size as i64;
                }
                Witness::Dynamic { location } => {
                    let new_dynamic_edge = {
                        let name = env.fresh_name();
                        env.define_variable(name, Type::integer(), Witness::trivial(8))
                    };
                    to_offset.offsets.instrs.push(Instr::Set {
                        target: new_dynamic_edge.clone(),
                        expr: Expr::Primitive(Primitive::Add, vec![dynamic_edge.clone(), location]),
                    });
                    offsets.insert(var.clone(), Offset::Dynamic(dynamic_edge));
                    offsets.insert(new_dynamic_edge.name.clone(), Offset::Trivial(static_edge));
                    dynamic_edge = new_dynamic_edge;
                    static_edge += 8;
                }
            }
        }
        static_edge += 8;
        to_offset.offsets.instrs.insert(
            0,
            Instr::Set {
                target: static_edge_var,
                expr: Expr::Literal(Literal::Integer(static_edge)),
            },
        );
    }

    Offsets { offsets }
}

fn arg_variables(args: &[Variable]) -> impl IntoIterator<Item = &String> {
    args.iter().rev().map(|var| &var.name)
}

fn body_variables(body: &Block) -> impl IntoIterator<Item = &String> {
    body.instrs.iter().flat_map(instr_variables)
}

fn instr_variables(instr: &Instr) -> impl IntoIterator<Item = &String> {
    let vars: Vec<_> = match instr {
        Instr::CallDirect { arguments, .. } => arguments.iter().collect(),
        Instr::Copy {
            target,
            value,
            witness,
        } => {
            if let Witness::Dynamic { location } = witness {
                vec![target, value, location]
            } else {
                vec![target, value]
            }
        }
        Instr::Destory { value, witness } => {
            if let Witness::Dynamic { location } = witness {
                vec![location, value]
            } else {
                vec![value]
            }
        }
        Instr::Set { target, expr } => {
            let mut vars: Vec<_> = match expr {
                Expr::Primitive(_, vars) => vars.iter().collect(),
                Expr::Literal(_) => vec![],
            };
            vars.push(target);
            vars
        }
    };
    vars.into_iter().map(|var| &var.name)
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Offset::Trivial(size) => write!(f, "{}", size),
            Offset::Dynamic(var) => write!(f, "{}", var),
        }
    }
}
