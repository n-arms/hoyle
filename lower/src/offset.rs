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
    pub stack_top: Variable,
}

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

    Offsets {
        offsets,
        stack_top: dynamic_edge,
    }
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
