use crate::builder;
use crate::env::Env;
use arena_alloc::General;
use ir::desugared::{
    Argument, Atom, Expr, Field, FieldDefinition, FunctionDefinition, Literal, Program, Statement,
    StructDefinition, Type,
};
use ir::qualified::{self, LocalTagSource};
use type_checker::extract::Typeable;

pub fn program<'old, 'new, 'names, 'meta>(
    to_desugar: qualified::Program<'old>,
    tags: LocalTagSource<'names>,
    env: &mut Env<'old, 'new, 'meta>,
    alloc: &General<'new>,
) -> Program<'new> {
    let mut program = builder::Program::new(tags);

    for def in to_desugar.definitions {
        definition(*def, &mut program, env, alloc);
    }

    program.build(alloc)
}

pub fn definition<'old, 'new, 'names, 'meta>(
    to_desugar: qualified::Definition<'old>,
    program: &mut builder::Program<'names, 'new>,
    env: &mut Env<'old, 'new, 'meta>,
    alloc: &General<'new>,
) {
    match to_desugar {
        qualified::Definition::Function(qualified::FunctionDefinition {
            name,
            generics,
            arguments,
            return_type,
            body,
            span,
        }) => {
            let mut desugared_arguments = Vec::new();

            for generic in generics {
                desugared_arguments.push(Argument {
                    name: generic.name.tag,
                    r#type: env.metadata_type,
                });
                env.bind_generic(
                    generic.name.tag,
                    Type::Any {
                        metadata: Expr::Atom(Atom::Variable(generic.name.tag)),
                    },
                );
            }

            let mut block = builder::Block::new(program.names);

            for argument in arguments {
                if let qualified::Pattern::Variable { name, .. } = argument.pattern {
                    desugared_arguments.push(Argument {
                        name: name.tag,
                        r#type: r#type(argument.r#type, env, alloc),
                    });
                } else {
                    todo!()
                }
            }

            let result = expr(body, &mut block, env, alloc);
            let function = FunctionDefinition {
                label: name.tag,
                arguments: alloc.alloc_slice_fill_iter(desugared_arguments),
                body: block.build(result, alloc),
            };
            program.with_function(function);
        }
        qualified::Definition::Struct { name, fields, .. } => {
            let desugared_fields =
                alloc.alloc_slice_fill_iter(fields.iter().map(|field| FieldDefinition {
                    name: field.name.identifier.tag,
                    r#type: r#type(field.name.r#type, env, alloc),
                }));
            let r#struct = StructDefinition {
                name: name.identifier.tag,
                fields: desugared_fields,
            };
            program.with_struct(r#struct);
        }
    }
}

pub fn expr<'old, 'new, 'names, 'meta>(
    to_desugar: qualified::Expr<'old>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'meta>,
    alloc: &General<'new>,
) -> Atom {
    match to_desugar {
        qualified::Expr::Variable { identifier, span } => {
            if let Some(function) = env.lookup_variable(identifier) {
                todo!()
            } else {
                Atom::Variable {
                    name: identifier.tag,
                }
            }
        }
        qualified::Expr::Literal {
            literal: Literal::Integer(int),
            span,
        } => Atom::Literal(Literal::Integer(
            int.parse()
                .expect("the parser should have caught improper integer literals"),
        )),
        qualified::Expr::Call {
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
        qualified::Expr::Operation {
            operator,
            arguments,
            ..
        } => todo!(),
        qualified::Expr::StructLiteral { name, fields, .. } => {
            let desugared_fields = alloc.alloc_slice_fill_iter(
                fields
                    .iter()
                    .map(|field| field_literal(*field, block, env, alloc)),
            );

            let struct_literal = Expr::Struct {
                fields: desugared_fields,
            };
            let result_name = block.fresh_tag();

            block.with_statement(Statement {
                variable: result_name,
                r#type: r#type(name.r#type, env, alloc),
                value: struct_literal,
            });
            Atom::Variable(result_name)
        }
        qualified::Expr::Block(qualified::Block {
            statements, result, ..
        }) => {
            for stmt in statements {
                statement(*stmt, block, env, alloc);
            }
            if let Some(result) = result {
                expr(*result, block, env, alloc)
            } else {
                todo!()
            }
        }
        qualified::Expr::Annotated {
            expr, annotation, ..
        } => todo!(),
        qualified::Expr::Case {
            predicate,
            branches,
            ..
        } => todo!(),
    }
}

pub fn bind_pattern<'old, 'new, 'names, 'meta>(
    to_bind: qualified::Pattern<'old>,
    bound_value: Atom,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'meta>,
    alloc: &General<'new>,
) {
    match to_bind {
        qualified::Pattern::Variable(name, _) => {
            block.with_statement(Statement {
                variable: name.identifier.tag,
                r#type: r#type(name.r#type, env, alloc),
                value: Expr::Atom(bound_value),
            });
        }
        qualified::Pattern::Struct { fields, .. } => {
            for field in fields {
                let field_tag = block.fresh_tag();
                block.with_statement(Statement {
                    variable: field_tag,
                    r#type: r#type(field.name.r#type, env, alloc),
                    value: Expr::FieldAccess {
                        r#struct: bound_value,
                        field: field.name.identifier.tag,
                    },
                });
            }
        }
    }
}

pub fn statement<'old, 'new, 'names, 'meta>(
    to_desugar: qualified::Statement<'old>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'meta>,
    alloc: &General<'new>,
) {
    match to_desugar {
        qualified::Statement::Let {
            left_side,
            right_side,
            ..
        } => {
            let right_side_result = expr(right_side, block, env, alloc);
            bind_pattern(left_side, right_side_result, block, env, alloc);
        }
        qualified::Statement::Raw(raw, _) => {
            expr(raw, block, env, alloc);
        }
    }
}

pub fn field_literal<'old, 'new, 'names, 'meta>(
    to_desugar: qualified::Field<'old>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'meta>,
    alloc: &General<'new>,
) -> Field {
    let name = to_desugar.name.identifier.tag;
    let value = expr(to_desugar.value, block, env, alloc);
    Field { name, value }
}

fn r#type<'old, 'new, 'meta>(
    to_desugar: qualified::Type<'old>,
    env: &mut Env<'old, 'new, 'meta>,
    alloc: &General<'new>,
) -> Type<'new> {
    match to_desugar {
        qualified::Type::Named { name, .. } => env.lookup_generic(name.tag),
        qualified::Type::Arrow {
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
