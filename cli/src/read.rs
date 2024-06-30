use std::io::{self, BufRead, Result, Write};

use lexer::{scan_tokens, Errors};
use tree::token::{self, List, Token};

pub enum ExitStatus {
    Okay,
    Error,
    Quit,
}

fn is_balanced<'a>(tokens: impl IntoIterator<Item = Token<'a>>) -> bool {
    let mut braces = 0i64;
    let mut parens = 0i64;
    let mut squares = 0i64;

    for token in tokens {
        match token.kind {
            token::Kind::LeftParen => parens += 1,
            token::Kind::RightParen => parens -= 1,
            token::Kind::LeftBrace => braces += 1,
            token::Kind::RightBrace => braces -= 1,
            token::Kind::LeftSquareBracket => squares += 1,
            token::Kind::RightSquareBracket => squares -= 1,
            _ => {}
        }
    }

    braces == 0 && parens == 0 && squares == 0
}

pub fn event_loop(name: &str, mut callback: impl FnMut(List, Errors) -> ExitStatus) -> Result<()> {
    let mut working_line = String::new();
    let stdin = io::stdin();

    println!("{}", name);

    print!("> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        working_line.push_str(&line?);

        let (tokens, errors) = scan_tokens(&working_line);

        if is_balanced(&tokens) {
            if let ExitStatus::Quit = callback(tokens, errors) {
                break;
            }

            working_line = String::new();
        }

        print!("> ");
        io::stdout().flush().unwrap();
    }

    Ok(())
}
