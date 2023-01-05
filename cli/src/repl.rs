use crate::read::ExitStatus;
use arena_alloc::*;
use bumpalo::Bump;
use ir::qualified::{TagSource, Type};
use ir::token::*;
use qualifier::definitions::Local;
use type_checker::{env::*, infer};

pub struct Repl<'a> {
    qualify: bool,
    type_check: bool,
    desugar: bool,

    definitions: Local<'a, 'a>,
    env: Env<'a, 'a>,

    ident_alloc: &'a Bump,
    tree1_alloc: &'a Bump,
    tree2_alloc: &'a Bump,
}

enum Command {
    Qualify(bool),
    Type(bool),
    Desugar(bool),
    Quit,
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
        [Token {
            kind: Kind::BinaryOperator(BinaryOperator::Cross),
            ..
        }, Token {
            kind: Kind::Identifier,
            span: Span {
                data: "desugar", ..
            },
        }] => Some(Command::Desugar(true)),
        [Token {
            kind: Kind::BinaryOperator(BinaryOperator::Dash),
            ..
        }, Token {
            kind: Kind::Identifier,
            span: Span {
                data: "desugar", ..
            },
        }] => Some(Command::Desugar(false)),
        [Token {
            kind: Kind::BinaryOperator(BinaryOperator::Dash | BinaryOperator::Cross),
            ..
        }, Token {
            kind: Kind::Identifier,
            span: Span { data: "quit", .. },
        }] => Some(Command::Quit),
        _ => None,
    }
}

impl<'a> Repl<'a> {
    pub fn new(ident_alloc: &'a Bump, tree1_alloc: &'a Bump, tree2_alloc: &'a Bump) -> Self {
        let tags = TagSource::default();
        let definitions = Local::new(1, tags.clone());
        let primitives = Primitives {
            int: Type::Named {
                name: definitions.lookup_type("int").unwrap(),
                span: None,
            },
            bool: Type::Named {
                name: definitions.lookup_type("bool").unwrap(),
                span: None,
            },
        };
        Self {
            qualify: true,
            type_check: true,
            desugar: false,

            env: Env::new(tags, primitives),
            definitions,

            ident_alloc,
            tree1_alloc,
            tree2_alloc,
        }
    }

    pub fn run(&mut self, tokens: List) -> ExitStatus {
        let ast_alloc = General::new(self.tree1_alloc);
        let ident_alloc = Interning::new(self.ident_alloc);

        let token_iter = tokens.into_iter();

        if let Some(command) = parse_command(token_iter.clone()) {
            match command {
                Command::Qualify(setting) => self.qualify = setting,
                Command::Type(setting) => self.type_check = setting,
                Command::Desugar(setting) => self.desugar = setting,
                Command::Quit => return ExitStatus::Quit,
            }
            return ExitStatus::Okay;
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
                return ExitStatus::Error;
            }
        };

        if !self.qualify {
            println!("{:#?}", raw_ast);
            return ExitStatus::Okay;
        }

        let qualified_alloc = General::new(self.tree2_alloc);

        let qualified_ast = match qualifier::qualify(
            raw_ast,
            &mut self.definitions,
            &ident_alloc,
            &qualified_alloc,
        ) {
            Ok(program) => program,
            Err(error) => {
                println!("error while qualifying program \n{:?}", raw_ast);
                println!("error: {:?}", error);
                return ExitStatus::Error;
            }
        };

        if !self.type_check {
            println!("{:#?}", qualified_ast);
            return ExitStatus::Okay;
        }

        let type_alloc = General::new(self.tree1_alloc);

        let typed_program =
            match infer::program(qualified_ast, &mut self.env, &ident_alloc, &type_alloc) {
                Ok(program) => program,
                Err(error) => {
                    println!("error while type checking program\n{:?}", qualified_ast);
                    println!("error: {:?}", error);
                    return ExitStatus::Error;
                }
            };

        println!("{:#?}", typed_program);

        ExitStatus::Okay
    }
}
