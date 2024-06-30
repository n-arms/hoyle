mod read;
mod repl;

use bumpalo::Bump;
use lower::lower;
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

fn run(tokens: token::List) -> read::ExitStatus {
    let parsed = match parser::parse(&tokens.into_iter().collect::<Vec<_>>()) {
        Ok(p) => p,
        Err(error) => {
            println!("parse error: {:?}", error);
            return read::ExitStatus::Error;
        }
    };
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
    println!("{}", sized);
    let bridged = lower::program(&sized);
    println!("{}", bridged);
    read::ExitStatus::Okay
}
