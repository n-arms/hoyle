use std::io::{self, BufRead, Result, Write};

use ir::token::{List, Token};
use lexer::{scan_tokens, Errors};

fn is_balanced<'a>(tokens: impl IntoIterator<Item = Token<'a>>) -> bool {
    let mut braces = 0i64;
    let mut parens = 0i64;
    let mut squares = 0i64;

    for token in tokens {
        match token.kind {
            ir::token::Kind::LeftParen => parens += 1,
            ir::token::Kind::RightParen => parens -= 1,
            ir::token::Kind::LeftBrace => braces += 1,
            ir::token::Kind::RightBrace => braces -= 1,
            ir::token::Kind::LeftSquareBracket => squares += 1,
            ir::token::Kind::RightSquareBracket => squares -= 1,
            _ => {}
        }
    }

    braces == 0 && parens == 0 && squares == 0
}

pub fn event_loop(name: &str, mut callback: impl FnMut(List, Errors)) -> Result<()> {
    let mut working_line = String::new();
    let stdin = io::stdin();

    println!("{}", name);

    print!("> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        working_line.push_str(&line?);

        let (tokens, errors) = scan_tokens(&working_line);

        if is_balanced(&tokens) {
            callback(tokens, errors);

            working_line = String::new();
        }

        print!("> ");
        io::stdout().flush().unwrap();
    }

    Ok(())
}
