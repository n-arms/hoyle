use core::fmt;

use im::HashSet;
use ir::{
    bridge::{Block, Convention, Expr, Function, Instr, Program, Struct, Value, Variable, Witness},
    name_source::NameSource,
};
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
    to_free: Vec<String>,
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
            Witness::Dynamic { location } => {
                self.defer_free(name.clone());
                source.pushln(&format!(
                    "void *{} = malloc(((_witness *) {}) -> size);",
                    name, location.name
                ))
            }
            Witness::Type => {
                source.pushln(&format!("char {}[sizeof(_witness)];", name));
            }
        }
        self.seen.insert(name);
    }

    fn defer_free(&mut self, name: String) {
        self.to_free.push(name);
    }

    fn free_list(&self) -> impl Iterator<Item = &str> {
        self.to_free.iter().rev().map(AsRef::as_ref)
    }
}

pub fn program(program: Program) -> Source {
    let mut source = Source::default();
    source.pushln(
        r#"#include <string.h>
#include <limits.h>
#include <stdlib.h>
typedef struct _witness {
  unsigned long long size;
  void (*move)(void *, void *, void *);
  void (*copy)(void *, void *, void *);
  void (*destroy)(void *, void *);
  void *extra;
} _witness;

void _move_F64(void *dest, void *src, void *extra) {
  memmove(dest, src, 8);
}

void _destroy_F64(void *dest, void *extra) {}

void F64(void *_result) {
  _witness *result = _result;
  result -> size = 8;
  result -> move = _move_F64;
  result -> copy = _move_F64;
  result -> destroy = _destroy_F64;
  result -> extra = NULL;
}

void _move_Bool(void *dest, void *src, void *extra) {
  memmove(dest, src, 8);
}

void _destroy_Bool(void *dest, void *extra) {}

void Bool(void *_result) {
  _witness *result = _result;
  result -> size = 8;
  result -> move = _move_Bool;
  result -> copy = _move_Bool;
  result -> destroy = _destroy_Bool;
  result -> extra = NULL;
}

struct _Closure {
  void *function;
  void *env;
};

void _copy_closure(void *dest, void *src, void *extra) {
  void *env = ((struct _Closure *) src) -> env;
  unsigned long long *counter = env;
  if (counter != NULL) {
    *counter += 1;
  }
  memmove(dest, src, sizeof(struct _Closure));
}

void _move_closure(void *dest, void *src, void *extra) {
  memmove(dest, src, sizeof(struct _Closure));
}

void _destroy_closure(void *src, void *extra) {
  void *env = ((struct _Closure *) src) -> env;
  unsigned long long *counter = env;
  if (counter != NULL) {
    counter -= 1;
    if (counter == 0) {
      free(env);
    }
  }
}

void _Closure(void *_result) {
  _witness *result = _result;
  result -> size = 8;
  result -> move = _move_closure;
  result -> copy = _copy_closure;
  result -> destroy = _destroy_closure;
  result -> extra = NULL;
}


void _move_type(void *dest, void *src) {
    memmove(dest, src, sizeof(_witness));
}

void _copy_type(void *dest, void *src) {
    _witness *typ = src;
    if (typ -> extra != NULL) {
        unsigned long long *counter = typ -> extra;
        if (*counter == ULONG_MAX) {
            exit(1);
        } else {
            *counter += 1;
        }
    }
    memmove(dest, src, sizeof(_witness));
}

void _destroy_type(void *src) {
    _witness *typ = src;
    if (typ -> extra != NULL) {
        unsigned long long *counter = typ -> extra;
        if (*counter == 0) {
            free(typ -> extra);
        } else {
            *counter -= 1;
        }
    }
}
"#,
    );

    for to_emit in program.structs.into_iter() {
        strukt(to_emit, &mut source);
    }
    for to_emit in program.functions.into_iter() {
        function(to_emit, &mut source);
    }
    source
}

fn copy_struct(to_emit: &Struct, source: &mut Source, names: &NameSource) {
    let struct_name = &to_emit.definition.name;
    source.pushln(&format!(
        "void _copy_{}(void *dest, void *src, void *extra) {{",
        struct_name
    ));
    source.with_inc(2, |source| {
        let mut bank = Bank::default();
        block(to_emit.builder.block.clone(), source, &mut bank, names);
        for field in &to_emit.builder.fields {
            source.pushln(&format!(
                "(((_witness *) {}) -> copy)(dest, src, ((_witness *) {}) -> extra);",
                field.name, field.name
            ))
        }
    });
    source.pushln("}");
}
fn move_struct(to_emit: &Struct, source: &mut Source, names: &NameSource) {
    let struct_name = &to_emit.definition.name;
    source.pushln(&format!(
        "void _move_{}(void *dest, void *src, void *extra) {{",
        struct_name
    ));
    source.with_inc(2, |source| {
        let mut bank = Bank::default();
        block(to_emit.builder.block.clone(), source, &mut bank, names);
        for field in &to_emit.builder.fields {
            source.pushln(&format!(
                "(((_witness *) {}) -> move)(dest, src, ((_witness *) {}) -> extra);",
                field.name, field.name
            ))
        }
    });
    source.pushln("}");
}
fn destroy_struct(to_emit: &Struct, source: &mut Source, names: &NameSource) {
    let struct_name = &to_emit.definition.name;
    source.pushln(&format!(
        "void _destroy_{}(void *dest, void *extra) {{",
        struct_name
    ));
    source.with_inc(2, |source| {
        let mut bank = Bank::default();
        block(to_emit.builder.block.clone(), source, &mut bank, names);
        for field in &to_emit.builder.fields {
            source.pushln(&format!(
                "(((_witness *) {}) -> destroy)(dest, ((_witness *) {}) -> extra);",
                field.name, field.name
            ))
        }
    });
    source.pushln("}");
}

fn get_field(to_emit: &Struct, field_index: usize, source: &mut Source, names: &NameSource) {
    let struct_name = &to_emit.definition.name;
    let field = &to_emit.definition.fields[field_index];
    source.pushln(&format!(
        "void _{struct_name}_get_{}(void *dest, void *src) {{",
        field.name
    ));
    source.with_inc(2, |source| {
        let mut bank = Bank::default();
        block(to_emit.builder.block.clone(), source, &mut bank, names);
        let offset_var = names.fresh_name();
        source.pushln(&format!("unsigned long long {offset_var} = 0;"));
        for field in to_emit.builder.fields.iter().take(field_index) {
            source.pushln(&format!("{offset_var} += ((_witness *) {});", field.name));
        }
        let field_witness = &to_emit.builder.fields[field_index].name;
        source.pushln(&format!("(((_witness *) {field_witness}) -> copy)(dest, ((char *) src) + {offset_var}, ((_witness *) {field_witness}) -> extra);"));
    });
    source.pushln("}");
}

fn strukt(to_emit: Struct, source: &mut Source) {
    let names = to_emit.builder.names.clone();
    copy_struct(&to_emit, source, &names);
    move_struct(&to_emit, source, &names);
    destroy_struct(&to_emit, source, &names);
    for field_index in 0..to_emit.definition.fields.len() {
        get_field(&to_emit, field_index, source, &names);
    }
    let struct_name = to_emit.definition.name;
    source.push(&format!("void {}(", struct_name));
    let mut first = true;
    for arg in &to_emit.builder.arguments {
        if first {
            first = false;
        } else {
            source.push(", ");
        }
        source.push(&format!("void *{}", arg.name.name));
    }
    source.pushln(") {");
    source.with_inc(2, |source| {
        let mut bank = Bank::default();
        block(to_emit.builder.block, source, &mut bank, &names);

        source.pushln(&format!("_witness *typ = _result;"));
        source.push("typ -> size = ");

        let mut first = true;
        for field in to_emit.builder.fields {
            if first {
                first = false
            } else {
                source.push(" + ");
            }
            source.push(&format!("((_witness *) {}) -> size", field.name));
        }
        source.pushln(";");
        source.pushln(&format!("typ -> move = _move_{};", struct_name));
        source.pushln(&format!("typ -> copy = _copy_{};", struct_name));
        source.pushln(&format!("typ -> destroy = _destroy_{};", struct_name));
        source.pushln("typ -> extra = NULL;");
    });
    source.pushln("}");
}

fn function(to_emit: Function, source: &mut Source) {
    let names = &to_emit.names;
    source.push(&format!("void {}(", to_emit.name));
    let mut bank = Bank::default();

    let mut first = true;
    for arg in to_emit.arguments {
        if !first {
            source.push(", ");
        } else {
            first = false;
        }
        source.push(&format!("void *{}", arg.name.name));
        bank.already_defined(arg.name.name);
    }
    source.pushln(") {");
    source.with_inc(2, |source| {
        block(to_emit.body, source, &mut bank, names);
        for to_free in bank.free_list() {
            source.pushln(&format!("free({});", to_free));
        }
    });
    source.pushln("}");
    source.pushln("");
}

fn block(to_emit: Block, source: &mut Source, bank: &mut Bank, names: &NameSource) {
    for to_emit in to_emit.instrs {
        instr(to_emit, source, bank, names);
    }
}

fn shift(dest: &Variable, src: &Value, source: &mut Source) {
    shift_to(&dest.name, src, source)
}

fn shift_to(dest: &str, src: &Value, source: &mut Source) {
    let src_name = &src.variable().name;
    match src.variable().witness.as_ref() {
        Witness::Trivial { size } => {
            source.pushln(&format!(
                "memmove({}, {}, {});",
                dest,
                src.variable().name,
                size
            ));
        }
        Witness::Dynamic { location } => {
            let function = match src {
                Value::Move(_) => "move",
                Value::Copy(_) => "copy",
            };
            source.pushln(&format!(
                "(((_witness *) {}) -> {})({}, {}, ((_witness *) {}) -> extra);",
                location.name, function, dest, src_name, location.name
            ))
        }
        Witness::Type => source.pushln(&format!("_copy_type({}, {});", dest, src_name)),
    }
}

fn instr(to_emit: Instr, source: &mut Source, bank: &mut Bank, names: &NameSource) {
    let var = &to_emit.target.name;
    bank.define(var, &to_emit.target.witness, source);
    match to_emit.value {
        Expr::Literal(to_emit) => {
            source.push(&match to_emit {
                Literal::Float(float) => format!("*(double *) {var} = {float}"),
                Literal::Integer(integer) => format!("*(signed long long *) {var} = {integer}ll"),
                Literal::Boolean(boolean) => format!(
                    "*(signed long long *) {var} = {}ll",
                    if boolean { "1" } else { "0" }
                ),
            });
            source.pushln(";");
        }
        Expr::Primitive(primitive, args) => {
            let type_name = {
                match &args[0].typ {
                    tree::typed::Type::Named { name, arguments } => {
                        assert!(arguments.is_empty());
                        match name.as_str() {
                            "F64" => "double",
                            "I64" => "signed long long",
                            _ => unimplemented!(),
                        }
                    }
                    tree::typed::Type::Generic { .. } => panic!(),
                    tree::typed::Type::Function { .. } => panic!(),
                }
            };
            source.push(&format!("*({type_name} *) {var} = "));
            match primitive {
                Primitive::Add => {
                    source.push(&format!(
                        "*(({type_name} *) {a}) + *(({type_name} *) {b})",
                        a = args[0].name,
                        b = args[1].name,
                    ));
                }
                Primitive::Sub => {
                    source.push(&format!(
                        "*(({type_name} *) {a}) - *(({type_name} *) {b})",
                        a = args[0].name,
                        b = args[1].name,
                    ));
                }
                Primitive::Mul => {
                    source.push(&format!(
                        "*(({type_name} *) {a}) * *(({type_name} *) {b})",
                        a = args[0].name,
                        b = args[1].name,
                    ));
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
                    let var = arg.value.variable();
                    let witness = &var.witness;
                    let name = if arg.convention == Convention::Out {
                        var.name.clone()
                    } else {
                        let name = names.fresh_name();
                        let dest = Variable::new(name.clone(), var.typ.clone(), *witness.clone());
                        bank.define(&name, &witness, source);
                        shift(&dest, &arg.value, source);
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
        Expr::Value(value) => {
            shift(&to_emit.target, &value, source);
        }
        Expr::Destroy => match to_emit.target.witness.as_ref() {
            Witness::Trivial { .. } => {}
            Witness::Dynamic { location } => source.pushln(&format!(
                "(((_witness *) {}) -> destroy)({}, ((_witness *) {}) -> extra);",
                location.name, var, location.name
            )),
            Witness::Type => source.pushln(&format!("_destroy_type({});", var)),
        },
        Expr::StructPack { arguments, .. } => {
            let offset_name = names.fresh_name();
            source.pushln(&format!("signed long long {} = 0;", offset_name));
            for arg in arguments {
                shift_to(
                    &format!("(((char *) {var}) + {offset_name})"),
                    &arg.value,
                    source,
                );
                match arg.value.variable().witness.as_ref() {
                    Witness::Trivial { size } => {
                        source.pushln(&format!("{} += {};", offset_name, size));
                    }
                    Witness::Dynamic { location } => {
                        source.pushln(&format!(
                            "{} += ((_witness *) {}) -> size;",
                            offset_name, location.name
                        ));
                    }
                    Witness::Type => unreachable!(),
                }
            }
        }
        Expr::If {
            predicate,
            true_branch,
            false_branch,
        } => {
            source.pushln(&format!(
                "if (*((signed long long *) {})) {{",
                predicate.name
            ));
            source.with_inc(2, |source| {
                block(true_branch, source, bank, names);
            });
            source.pushln("} else {");
            source.with_inc(2, |source| {
                block(false_branch, source, bank, names);
            });
            source.pushln("}");
        }
        Expr::Unpack {
            value,
            field,
            struct_name,
            type_arguments,
        } => {
            let struct_var = &value.name;
            source.push(&format!("_{struct_name}_get_{field}({var}, {struct_var}"));
            for arg in type_arguments {
                source.push(&format!(", {}", arg.name));
            }
            source.pushln(");");
        }
        Expr::MakeClosure {
            function,
            env,
            witness,
        } => {
            let alloc = names.fresh_name();
            source.pushln(&format!(
                "void *{alloc} = malloc(sizeof(unsigned long long) + sizeof(_witness) + ((_witness *) {}) -> size);",
                witness.variable().name
            ));
            source.pushln(&format!("*(unsigned long long *) {alloc} = 0ull;"));
            shift_to(
                &format!("(((char *) {alloc}) + sizeof(unsigned long long))"),
                &witness,
                source,
            );
            shift_to(
                &format!("(((char *) {alloc}) + sizeof(_witness) + sizeof(unsigned long long))"),
                &env,
                source,
            );
            source.pushln(&format!("*(void **){var} = {function};"));
            source.pushln(&format!("*((void **){var} + 1) = {alloc};"));
        }
    }
}
