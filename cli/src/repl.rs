use crate::read::ExitStatus;
use arena_alloc::*;
use bumpalo::Bump;
use ir::qualified::{self, LocalTagSource, Primitives, TagSource, Type};
use ir::token::*;
use qualifier::definitions::Local;
use type_checker::{env::*, infer};

pub struct Repl<'a> {
    qualify: bool,
    type_check: bool,
    desugar: bool,

    definitions: Local<'a, 'a, 'a>,
    env: Env<'a, 'a, 'a>,

    local_tags: LocalTagSource<'a>,
    primitives: Primitives<'a>,
    metadata_type: ir::desugared::Type<'a>,

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
        let tags = ident_alloc.alloc(TagSource::default());
        let primitives = Primitives {
            int: tags.fresh_identifier("int", 0),
            bool: tags.fresh_identifier("bool", 0),
        };
        let metadata_type = ir::desugared::Type::Named {
            name: tags.fresh_tag(0),
        };
        let definitions = Local::new(LocalTagSource::new(1, tags), primitives);
        Self {
            qualify: true,
            type_check: true,
            desugar: false,

            env: Env::new(LocalTagSource::new(0, tags), primitives),
            definitions,

            local_tags: LocalTagSource::new(1, tags),
            primitives,
            metadata_type,

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

        if !self.desugar {
            println!("{:#?}", typed_program);
            return ExitStatus::Okay;
        }

        let desugar_alloc = General::new(self.tree2_alloc);

        let metadata = metadata::program(typed_program, self.local_tags, &desugar_alloc);

        let mut env = desugar::env::Env::new(&metadata, self.primitives, self.metadata_type);

        let desugared_program =
            desugar::desugar::program(typed_program, self.local_tags, &mut env, &desugar_alloc);

        println!("{:#?}", desugared_program);

        ExitStatus::Okay
    }
}
