use arena_alloc::*;
use bumpalo::Bump;
use infinite_iterator::InfiniteIterator;
use lexer::scan_tokens;
use parser::parser::program;
use type_checker::{env::*, infer};

use std::{
    io::{self, BufRead},
    ops::RangeFrom,
};

struct IdSource<'a> {
    counter: usize,
    alloc: Alloc<'a>,
}

impl<'a> IdSource<'a> {
    fn new(alloc: Alloc<'a>) -> Self {
        Self { alloc, counter: 0 }
    }
}

impl<'a> Iterator for IdSource<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_infinite())
    }
}
impl<'a> InfiniteIterator for IdSource<'a> {
    fn next_infinite(&mut self) -> <Self as Iterator>::Item {
        self.counter += 1;
        self.alloc.str(&self.counter.to_string())
    }
}

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

        let mut text = tokens.into_iter().peekable();

        let program = match program(&mut text, &alloc, &interner) {
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

        println!("{:?}", program);

        /*
        let type_bump = Bump::new();
        let type_alloc = Alloc::new(&type_bump);
        let type_env = Env::default();
        let mut fresh = Fresh::new(IdSource::new(type_alloc));
        let typed_program = infer::program(&program, type_env, &mut fresh, type_alloc).unwrap();

        println!("{:?}", typed_program)
        */
    }
}
