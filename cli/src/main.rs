mod read;
mod repl;
mod test;

use std::fs;

use bumpalo::Bump;
use lower::lower;
use read::test_loop;
use sizer::sizer;
use tree::token;
fn main() {
    read::event_loop("Welcome to the Hoyle repl", |tokens, errors| {
        if errors.success() {
            run(tokens)
        } else {
            println!("error while lexing: {:?}", errors);
            read::ExitStatus::Error
        }
    })
    .unwrap()
}

/*
fn main() {
    test_loop(
        r#"
            func id[t](x: t): t = x
            func a(): F64 = id(3)
        "#,
        run,
    )
}
*/

fn run(tokens: token::List) -> read::ExitStatus {
    println!("tokens: {:?}", tokens);
    let parsed = match parser::parse(&tokens.into_iter().collect::<Vec<_>>()) {
        Ok(p) => p,
        Err(error) => {
            println!("parse error: {:?}", error);
            return read::ExitStatus::Error;
        }
    };
    println!("parsed");
    let typed = match type_checker::infer::program(&parsed) {
        Ok(t) => t,
        Err(error) => {
            println!("type error: {:?}", error);
            return read::ExitStatus::Error;
        }
    };
    println!("okay");
    let passed = type_passing::pass::program(&typed);
    let sized = sizer::program(&passed);
    println!("sized");
    println!("{}", sized);
    println!("printed size");
    let bridged = lower::program(&sized);
    println!("{}", bridged);
    let c_source = emit::program(bridged);
    fs::write("gen/out.c", c_source.to_string()).unwrap();
    read::ExitStatus::Okay
}
