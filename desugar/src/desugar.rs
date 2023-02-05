use crate::builder;
use crate::env::Env;
use arena_alloc::General;
use ir::desugared::{Argument, Atom, Expr, FunctionDefinition, Literal, Program, Statement, Type};
use ir::qualified::{self, LocalTagSource};
use ir::typed;
use type_checker::extract::Typeable;

pub fn program<'old, 'new, 'names, 'ident, 'meta>(
    to_desugar: typed::Program<'old, 'ident>,
    tags: LocalTagSource<'names>,
    env: &mut Env<'old, 'new, 'ident, 'meta>,
    alloc: &General<'new>,
) -> Program<'new> {
    let mut program = builder::Program::new(tags);

    for def in to_desugar.definitions {
        definition(*def, &mut program, env, alloc);
    }

    program.build(alloc)
}

pub fn definition<'old, 'new, 'names, 'ident, 'meta>(
    to_desugar: typed::Definition<'old, 'ident>,
    program: &mut builder::Program<'names, 'new>,
    env: &mut Env<'old, 'new, 'ident, 'meta>,
    alloc: &General<'new>,
) {
    match to_desugar {
        ir::ast::Definition::Function {
            name,
            generics,
            arguments,
            body,
            ..
        } => {
            let mut desugared_arguments = Vec::new();

            for generic in generics {
                desugared_arguments.push(Argument {
                    name: generic.identifier.tag,
                    r#type: env.metadata_type,
                });
                env.bind_generic(
                    generic.identifier.tag,
                    Type::Any {
                        metadata: Expr::Atom(Atom::Variable(generic.identifier.tag)),
                    },
                );
            }

            let mut block = builder::Block::new(program.names);

            for argument in arguments {
                if let typed::Pattern::Variable(identifier, ..) = argument.pattern {
                    desugared_arguments.push(Argument {
                        name: identifier.identifier.tag,
                        r#type: r#type(argument.type_annotation, env, alloc),
                    });
                } else {
                    todo!()
                }
            }

            let result = expr(body, &mut block, env, alloc);
            let function = FunctionDefinition {
                label: name.identifier.tag,
                arguments: alloc.alloc_slice_fill_iter(desugared_arguments),
                body: block.build(result, alloc),
            };
            program.with_function(function);
        }
        ir::ast::Definition::Struct { name, fields, .. } => {}
    }
}

pub fn expr<'old, 'new, 'names, 'ident, 'meta>(
    to_desugar: typed::Expr<'old, 'ident>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'ident, 'meta>,
    alloc: &General<'new>,
) -> Atom {
    match to_desugar {
        ir::ast::Expr::Variable(variable, _) => {
            if let Some(function) = env.lookup_variable(variable.identifier) {
                todo!()
            } else {
                Atom::Variable(variable.identifier.tag)
            }
        }
        ir::ast::Expr::Literal(ir::ast::Literal::Integer(int), _) => {
            Atom::Literal(Literal::Integer(
                int.parse()
                    .expect("the parser should have caught improper integer literals"),
            ))
        }
        ir::ast::Expr::Call {
            function,
            arguments,
            ..
        } => {
            let desugared_function = expr(*function, block, env, alloc);
            let desugared_arguments = alloc
                .alloc_slice_fill_iter(arguments.iter().map(|arg| expr(*arg, block, env, alloc)));
            let call = Expr::Call {
                function: alloc.alloc(desugared_function),
                arguments: desugared_arguments,
            };
            let result_name = block.fresh_tag();
            let call_result = if let qualified::Type::Arrow { return_type, .. } =
                function.extract(&env.primitives)
            {
                r#type(*return_type, env, alloc)
            } else {
                unreachable!()
            };
            block.with_statement(Statement {
                variable: result_name,
                r#type: call_result,
                value: call,
            });
            Atom::Variable(result_name)
        }
        ir::ast::Expr::Operation {
            operator,
            arguments,
            ..
        } => todo!(),
        ir::ast::Expr::StructLiteral { name, fields, .. } => todo!(),
        ir::ast::Expr::Block(_) => todo!(),
        ir::ast::Expr::Annotated {
            expr, annotation, ..
        } => todo!(),
        ir::ast::Expr::Case {
            predicate,
            branches,
            ..
        } => todo!(),
    }
}

fn r#type<'old, 'new, 'ident, 'meta>(
    to_desugar: qualified::Type<'old, 'ident>,
    env: &mut Env<'old, 'new, 'ident, 'meta>,
    alloc: &General<'new>,
) -> Type<'new> {
    match to_desugar {
        ir::ast::Type::Named { name, .. } => env.lookup_generic(name.tag),
        ir::ast::Type::Arrow {
            arguments,
            return_type,
            ..
        } => Type::Function {
            arguments: alloc
                .alloc_slice_fill_iter(arguments.iter().map(|arg| r#type(*arg, env, alloc))),
            result: alloc.alloc(r#type(*return_type, env, alloc)),
        },
    }
}
