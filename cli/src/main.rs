mod read;
mod repl;

use arena_alloc::*;
use bumpalo::Bump;
use ir::typed::Type;
use lexer::scan_tokens;
use parser::program::program;
use qualifier::definitions::{Definitions, GlobalDefinitions};
use type_checker::{env::*, infer};

use std::io::{self, BufRead};

fn extract_primitives<'old, 'new, 'ident>(
    defs: Definitions<'old, 'ident>,
) -> Primitives<'new, 'ident> {
    Primitives {
        int: Type::Named {
            name: defs.lookup_type("int").unwrap(),
            span: None,
        },
        bool: Type::Named {
            name: defs.lookup_type("bool").unwrap(),
            span: None,
        },
    }
}

fn main() {
    let bump = Bump::new();
    let mut repl = repl::Repl::new(&bump, &bump, &bump);
    read::event_loop("Welcome to the Hoyle repl", |tokens, errors| {
        if errors.success() {
            repl.run(tokens);
        } else {
            println!("error while lexing: {:?}", errors)
        }
    })
    .unwrap()
}
