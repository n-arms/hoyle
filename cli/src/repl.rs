use crate::read::event_loop;
use arena_alloc::*;
use bumpalo::Bump;
use ir::token::*;
use ir::typed::Type;
use lexer::scan_tokens;
use parser;
use qualifier::definitions::{Definitions, GlobalDefinitions};
use std::cell::RefCell;
use std::rc::Rc;
use type_checker::{env::*, infer};

pub struct Repl<'a> {
    qualify: bool,
    type_check: bool,

    definitions: Rc<RefCell<GlobalDefinitions<'a, 'a>>>,
    env: Env<'a, 'a>,

    ident_alloc: &'a Bump,
    tree1_alloc: &'a Bump,
    tree2_alloc: &'a Bump,
}

enum Command {
    Qualify(bool),
    Type(bool),
}

fn parse_command<'a>(tokens: impl IntoIterator<Item = Token<'a>>) -> Option<Command> {
    match tokens.into_iter().collect::<Vec<_>>().as_slice() {
        [Token {
            kind: Kind::BinaryOperator(BinaryOperator::Cross),
            ..
        }, Token {
            kind: Kind::Identifier,
            span: Span {
                data: "qualify", ..
            },
        }] => Some(Command::Qualify(true)),
        [Token {
            kind: Kind::BinaryOperator(BinaryOperator::Dash),
            ..
        }, Token {
            kind: Kind::Identifier,
            span: Span {
                data: "qualify", ..
            },
        }] => Some(Command::Qualify(false)),
        [Token {
            kind: Kind::BinaryOperator(BinaryOperator::Cross),
            ..
        }, Token {
            kind: Kind::Identifier,
            span: Span { data: "type", .. },
        }] => Some(Command::Type(true)),
        [Token {
            kind: Kind::BinaryOperator(BinaryOperator::Dash),
            ..
        }, Token {
            kind: Kind::Identifier,
            span: Span { data: "type", .. },
        }] => Some(Command::Type(false)),
        _ => None,
    }
}

impl<'a> Repl<'a> {
    pub fn new(ident_alloc: &'a Bump, tree1_alloc: &'a Bump, tree2_alloc: &'a Bump) -> Self {
        let defs = GlobalDefinitions::default();
        let primitives = Primitives {
            int: Type::Named {
                name: defs.lookup_type("int").unwrap(),
                span: None,
            },
            bool: Type::Named {
                name: defs.lookup_type("bool").unwrap(),
                span: None,
            },
        };
        Self {
            qualify: true,
            type_check: true,

            definitions: Rc::new(RefCell::new(defs)),
            env: Env::new(primitives),

            ident_alloc,
            tree1_alloc,
            tree2_alloc,
        }
    }

    pub fn run(&mut self, tokens: List) {
        let ast_alloc = General::new(&self.tree1_alloc);
        let ident_alloc = Interning::new(&self.ident_alloc);

        let token_iter = tokens.into_iter();

        match parse_command(token_iter.clone()) {
            Some(command) => {
                match command {
                    Command::Qualify(setting) => self.qualify = setting,
                    Command::Type(setting) => self.type_check = setting,
                }
                return;
            }
            None => {}
        };

        let mut text = token_iter.clone().peekable();

        let raw_ast = match parser::parse(&mut text, &ast_alloc, &ident_alloc) {
            Ok(program) => program,
            Err(e) => {
                println!(
                    "error while parsing tokens {:?}",
                    token_iter.collect::<Vec<_>>()
                );
                println!("{:?}", e);
                return;
            }
        };

        if !self.qualify {
            println!("{:#?}", raw_ast);
            return;
        }

        let qualified_alloc = General::new(&self.tree2_alloc);
        let mut defs = Definitions::new(1, Rc::clone(&self.definitions));

        let qualified_ast =
            match qualifier::qualify(raw_ast, &mut defs, &ident_alloc, &qualified_alloc) {
                Ok(program) => program,
                Err(error) => {
                    println!("error while qualifying program \n{:?}", raw_ast);
                    println!("error: {:?}", error);
                    return;
                }
            };

        if !self.type_check {
            println!("{:#?}", qualified_ast);
            return;
        }

        let type_alloc = General::new(&self.tree1_alloc);

        let typed_program =
            match infer::program(qualified_ast, &mut self.env, &ident_alloc, &type_alloc) {
                Ok(program) => program,
                Err(error) => {
                    println!("error while type checking program\n{:?}", qualified_ast);
                    println!("error: {:?}", error);
                    return;
                }
            };

        println!("{:#?}", typed_program);
    }
}
