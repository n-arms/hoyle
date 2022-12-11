use bumpalo::Bump;
use lexer::scan_tokens;
use parser::{alloc::*, parser::program};

use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines().map(Result::unwrap) {
        let (tokens, token_errors) = scan_tokens(&line);

        if !token_errors.success() {
            println!("{:?}", token_errors);
            continue;
        }
        let ident = Bump::new();
        let ast = Bump::new();
        let alloc = General::new(&ident, &ast);

        let mut text = tokens.into_iter().peekable();

        let program = match program(&mut text, &alloc) {
            Ok(Ok(program)) => program,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
            Ok(Err(e)) => {
                println!("{:?}", e);
                continue;
            }
        };

        println!("{:#?}", program)
    }
}
