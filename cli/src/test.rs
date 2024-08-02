use core::fmt;
use std::{fs, process};

use lexer::scan_tokens;
use lower::lower;
use sizer::sizer;

fn to_c(text: &str) -> String {
    let (tokens, errors) = scan_tokens(text);
    assert!(errors.success());

    println!("tokens: {:?}", tokens);
    let parsed = parser::parse(&tokens.into_iter().collect::<Vec<_>>()).unwrap();
    println!("parsed");
    let typed = type_checker::infer::program(&parsed).unwrap();
    println!("okay");
    let passed = type_passing::pass::program(&typed);
    let sized = sizer::program(&passed);
    println!("sized");
    println!("{}", sized);
    println!("printed size");
    let bridged = lower::program(&sized);
    println!("{}", bridged);
    let c_source = emit::program(bridged);
    c_source.to_string()
}

fn run_double_func(c_program: &str, double_func: &str, formatter: &str, cast_to: &str) -> String {
    let seed = fastrand::u64(u64::MIN..=u64::MAX);
    let path = format!("./target/gen/{double_func}{seed}/");
    fs::DirBuilder::new().recursive(true).create(&path).unwrap();
    let prefix = fs::canonicalize(path).unwrap();

    let in_working = |tail: &str| {
        let mut other = prefix.clone();
        other.push(tail);
        other
    };
    fs::write(in_working("out.c"), c_program).unwrap();
    fs::write(
        in_working("main.c"),
        format!(
            r#"#include "out.c"
#include <stdio.h>

int main() {{
  {cast_to} x;
  {double_func}(&x);
  printf("{formatter}", ({cast_to}) x);
}}"#
        ),
    )
    .unwrap();
    let result = process::Command::new("gcc")
        .current_dir(&prefix)
        .arg("main.c")
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success();
    assert!(result, "emitted c code doesn't compile");
    let child = process::Command::new("./a.out")
        .current_dir(&prefix)
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn run<O: OutputType + fmt::Display>(text: &str, main: &str, expected: O) {
    let value = run_double_func(&to_c(text), main, O::formatter(), O::cast_to());
    assert!(
        expected.equals(value.clone()),
        "{main}() = {value} is not {expected}",
    )
}

trait OutputType {
    fn equals(&self, other: String) -> bool;
    fn formatter() -> &'static str;
    fn cast_to() -> &'static str;
}

impl OutputType for f64 {
    fn equals(&self, other: String) -> bool {
        let other: f64 = other
            .parse()
            .expect(&format!("could not parse {} as float", other));
        (other - self).abs() < 0.0001
    }

    fn formatter() -> &'static str {
        "%lf"
    }

    fn cast_to() -> &'static str {
        "double"
    }
}

impl OutputType for bool {
    fn equals(&self, other: String) -> bool {
        let other: i64 = other
            .parse()
            .expect(&format!("could not parse {} as bool", other));
        if *self {
            other == 1
        } else {
            other == 0
        }
    }

    fn formatter() -> &'static str {
        "%ld"
    }

    fn cast_to() -> &'static str {
        "signed long long"
    }
}

#[test]
fn literal() {
    run(
        r#"
        func literal(): F64 = 3
        "#,
        "literal",
        3.,
    )
}

#[test]
fn mono_id() {
    run(
        r#"
        func id(x: F64): F64 = x
        func mono_id(): F64 = id(3)
        "#,
        "mono_id",
        3.,
    )
}

#[test]
fn poly_id() {
    run(
        r#"
        func id[t](x: t): t = x
        func poly_id(): F64 = id(3)
        "#,
        "poly_id",
        3.,
    )
}

#[test]
fn chained_poly_id() {
    run(
        r#"
        func id[t](x: t): t = x
        func chained_poly_id(): F64 = id(id(3))
        "#,
        "chained_poly_id",
        3.,
    )
}

#[test]
fn chained_poly_id_literal() {
    run(
        r#"
        func id[t](x: t): t = x
        func literal(): F64 = 3
        func chained_poly_id_literal(): F64 = id(id(literal()))
        "#,
        "chained_poly_id_literal",
        3.,
    )
}

#[test]
fn first() {
    run(
        r#"
        func first_of[a, b](x: a, y: b): a = x
        func first(): F64 = first_of(3, 4)
        "#,
        "first",
        3.,
    )
}

#[test]
fn r#let() {
    run(
        r#"
        func let_(): F64 = {
            let x = 3;
            x
        }
        "#,
        "let_",
        3.,
    )
}

#[test]
fn poly_let() {
    run(
        r#"
        func id[t](x: t): t = {
            let y = x;
            y
        }
        func poly_let(): F64 = id(3)
        "#,
        "poly_let",
        3.,
    )
}

#[test]
fn multiplication() {
    run(
        r#"
        func times_and_2(x: F64, y: F64): F64 = x * y * 2
        func multiplication(): F64 = times_and_2(2 * 3, 4) * 5
        "#,
        "multiplication",
        240.,
    )
}

#[test]
fn bedmas() {
    run(
        r#"
        func bedmas(): F64 = 3 * 4 + 5 * 6
        "#,
        "bedmas",
        42.,
    )
}

// TODO: implement a better test for struct packing
#[test]
fn struct_pack() {
    run(
        r#"
        struct Box {
            x: F64
        }
        func struct_pack(): Box = Box { x: 3 }
        "#,
        "struct_pack",
        3.,
    )
}

#[test]
fn chained_poly_struct_pack() {
    run(
        r#"
        struct Box {
            x: F64
        }
        func id[t](x: t): t = x
        func struct_pack(): Box = id(id(Box { x: 3 }))
        "#,
        "struct_pack",
        3.,
    )
}

#[test]
fn bool_literal() {
    run(
        r#"
        func bool_literal(): Bool = True
        "#,
        "bool_literal",
        true,
    )
}

#[test]
fn chained_poly_bool_literal() {
    run(
        r#"
        func id[t](x: t): t = x
        func bool_literal(): Bool = id(id(True))
        "#,
        "bool_literal",
        true,
    )
}

#[test]
fn if_false() {
    run(
        r#"
        func if_false(): F64 = if False then 4 else 3
        "#,
        "if_false",
        3.,
    )
}

#[test]
fn if_let() {
    run(
        r#"
        func nested_if_let(): F64 = {
            let q = True;
            let p = q;
            let r = if p then {
                let a = 3;
                let b = a;
                b
            } else {
                let c = 4;
                let d = c;
                let e = c;
                e
            };
            r
        }
        "#,
        "nested_if_let",
        3.,
    )
}

#[test]
fn nested_if() {
    run(
        r#"
        func nested_if(): F64 = if if True then False else True 
            then if False then 3 else 4 
            else if True then 6 else 5
        "#,
        "nested_if",
        6.,
    )
}

#[test]
fn nested_if_result() {
    run(
        r#"
        func nested_if_result(): F64 = if True 
            then if False then 3 else 4 
            else if True then 6 else 5
        "#,
        "nested_if_result",
        4.,
    )
}
