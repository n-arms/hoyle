use arena_alloc::*;
use bumpalo::Bump;
use lexer::scan_tokens;
use parser::program::program;
use qualifier::definitions::Definitions;
use type_checker::{env::*, infer};

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
        let alloc = General::new(&ast);
        let interner = Interning::new(&ident);

        let token_iter = tokens.into_iter();
        let mut text = token_iter.clone().peekable();

        let raw_program = match program(&mut text, &alloc, &interner) {
            Ok(Ok(program)) => program,
            Err(e) => {
                println!("{:?}", token_iter.collect::<Vec<_>>());
                println!("{:?}", e);
                continue;
            }
            Ok(Err(e)) => {
                println!("{:?}", e);
                continue;
            }
        };

        println!("{:?}", raw_program);

        let qualified_ast_bump = Bump::new();
        let qualifying_alloc = General::new(&qualified_ast_bump);
        let mut defs = Definitions::default();

        let qualified_program = match qualifier::qualifier::program(
            raw_program,
            &mut defs,
            &interner,
            &qualifying_alloc,
        ) {
            Ok(qp) => qp,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };

        println!("{:?}", qualified_program);

        let type_bump = Bump::new();
        let type_alloc = General::new(&type_bump);
        let mut type_env = Env::default();
        let typed_program =
            infer::program(qualified_program, &mut type_env, &interner, &type_alloc).unwrap();

        println!("{:#?}", typed_program)
    }
}
