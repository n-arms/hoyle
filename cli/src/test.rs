use std::{
    fs,
    io::Read,
    process::{self, Stdio},
};

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
    let (passed, builders) = type_passing::pass::program(&typed);
    let (sized, builders) = sizer::program(&passed, &builders);
    println!("sized");
    println!("{}", sized);
    println!("printed size");
    let (bridged, mut envs, mut struct_envs) = lower::program(&sized, &builders);
    println!("{}", bridged);
    let c_source = emit::program(bridged, &mut envs, &mut struct_envs);
    c_source.to_string()
}

fn run_double_func(c_program: &str, double_func: &str) -> f64 {
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
  double x;
  {double_func}(&x);
  printf("%lf", x);
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
    let string = String::from_utf8(output.stdout).unwrap();
    string
        .parse()
        .expect(&format!("couldn't parse string {:?}", string))
}

fn run(text: &str, main: &str, expected: f64) {
    assert!(
        (run_double_func(&to_c(text), main) - expected).abs() < 0.0001,
        "{main}() != {expected}",
    )
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
