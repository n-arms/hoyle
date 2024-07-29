use core::fmt;

use im::HashSet;
use ir::bridge::{Block, Convention, Expr, Function, Instr, Program, Variable, Witness};
use lower::env::Env;
use tree::{sized::Primitive, typed::Literal, String};

type StdString = std::string::String;

#[derive(Clone, Default)]
pub struct Source {
    buffer: StdString,
    current_line: StdString,
    indent: usize,
}

impl Source {
    pub fn push(&mut self, text: &str) {
        self.current_line.push_str(text);
    }

    pub fn pushln(&mut self, text: &str) {
        self.buffer.reserve(text.len() + 1 + self.indent);
        self.buffer.push('\n');
        for _ in 0..self.indent {
            self.buffer.push(' ');
        }
        self.buffer.push_str(&self.current_line);
        self.buffer.push_str(text);
        self.current_line.clear();
    }

    pub fn inc(&mut self, indent: usize) {
        self.indent += indent;
    }

    pub fn dec(&mut self, indent: usize) {
        self.indent -= indent;
    }

    pub fn with_inc<T>(&mut self, indent: usize, callback: impl FnOnce(&mut Self) -> T) -> T {
        self.inc(indent);
        let res = callback(self);
        self.dec(indent);
        return res;
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.buffer)
    }
}

#[derive(Default)]
pub struct Bank {
    seen: HashSet<String>,
}

impl Bank {
    pub fn already_defined(&mut self, name: String) {
        self.seen.insert(name);
    }

    pub fn define(&mut self, name: &str, witness: &Witness, source: &mut Source) {
        if !self.seen.contains(name) {
            self.define_unchecked(String::from(name), witness, source);
        }
    }

    fn define_unchecked(&mut self, name: String, witness: &Witness, source: &mut Source) {
        match witness {
            Witness::Trivial { size } => source.pushln(&format!("char {}[{}];", name, size)),
            Witness::Dynamic { location } => source.pushln(&format!(
                "void *{} = malloc(((_witness *) {}) -> size);",
                name, location.name
            )),
        }
        self.seen.insert(name);
    }
}

pub fn program(program: Program, envs: &mut [Env]) -> Source {
    let mut source = Source::default();
    source.pushln(
        r#"#include <string.h>
typedef struct _witness {
  void (*move)(void *, void *, void *);
  void *extra;
} _witness;
"#,
    );
    for (to_emit, env) in program.functions.into_iter().zip(envs) {
        function(to_emit, &mut source, env);
    }
    source
}

fn function(to_emit: Function, source: &mut Source, env: &mut Env) {
    source.push(&format!("void {}(", to_emit.name));
    let mut bank = Bank::default();

    let mut first = true;
    for arg in to_emit.arguments {
        if !first {
            source.push(", ");
        } else {
            first = false;
        }
        source.push(&format!("void *{}", arg.name));
        bank.already_defined(arg.name);
    }
    source.pushln(") {");
    source.with_inc(2, |source| block(to_emit.body, source, &mut bank, env));
    source.pushln("}");
    source.pushln("");
}

fn block(to_emit: Block, source: &mut Source, bank: &mut Bank, env: &mut Env) {
    for to_emit in to_emit.instrs {
        instr(to_emit, source, bank, env);
    }
}

fn copy(dest: &Variable, src: &Variable, witness: &Witness, source: &mut Source) {
    match witness {
        Witness::Trivial { size } => {
            source.pushln(&format!("memmove({}, {}, {});", dest.name, src.name, size));
        }
        Witness::Dynamic { location } => source.pushln(&format!(
            "(((_witness *) {}) -> copy)({}, {}, ((_witness *) {}) -> extra);",
            location.name, dest.name, src.name, location.name
        )),
    }
}

fn instr(to_emit: Instr, source: &mut Source, bank: &mut Bank, env: &mut Env) {
    let var = &to_emit.target.name;
    let witness = env.lookup_witness(var);
    bank.define(var, &witness, source);
    match to_emit.value {
        Expr::Literal(to_emit) => {
            source.push(&match to_emit {
                Literal::Float(float) => format!("*(double *) {var} = {float}"),
                Literal::Integer(integer) => format!("*(signed long long *) {var} = {integer}ll"),
            });
            source.pushln(";");
        }
        Expr::Primitive(primitive, args) => {
            source.push(&format!("*{} = ", var));
            match primitive {
                Primitive::Add => {
                    variable(&args[0].clone(), source);
                    source.push(" + ");
                    variable(&args[1].clone(), source);
                }
                Primitive::Sub => {
                    variable(&args[0].clone(), source);
                    source.push(" - ");
                    variable(&args[1].clone(), source);
                }
            };
            source.pushln(";");
        }
        Expr::CallDirect {
            function,
            arguments,
        } => {
            let emitted_args: Vec<_> = arguments
                .iter()
                .map(|arg| {
                    let witness = env.lookup_witness(&arg.value.name);
                    let name = if arg.convention == Convention::Out {
                        arg.value.name.clone()
                    } else {
                        let name = env.fresh_name();
                        let dest = env.define_variable(
                            name.clone(),
                            arg.value.typ.clone(),
                            witness.clone(),
                        );
                        bank.define(&name, &witness, source);
                        copy(&dest, &arg.value, &witness, source);
                        name
                    };
                    name
                })
                .collect();
            source.push(&format!("{}(", function));
            let mut first = true;
            for arg in emitted_args {
                if !first {
                    source.push(", ");
                } else {
                    first = false;
                }
                source.push(&arg);
            }
            source.pushln(");");
        }
        Expr::Move {
            source: src,
            witness,
        } => match witness {
            Witness::Trivial { size } => {
                source.pushln(&format!("memmove({}, {}, {});", var, src.name, size));
            }
            Witness::Dynamic { location } => source.pushln(&format!(
                "(((_witness *) {}) -> move)({}, {}, ((_witness *) {}) -> extra);",
                location.name, var, src.name, location.name
            )),
        },
        Expr::Copy {
            source: src,
            witness,
        } => copy(&to_emit.target, &src, &witness, source),
        Expr::Destroy { witness } => match witness {
            Witness::Trivial { .. } => {}
            Witness::Dynamic { location } => source.pushln(&format!(
                "(((_witness *) {}) -> destory)({}, ((_witness *) {}) -> extra);",
                location.name, var, location.name
            )),
        },
    }
}

fn variable(to_emit: &Variable, source: &mut Source) {
    source.push(&format!("*{}", to_emit.name))
}
