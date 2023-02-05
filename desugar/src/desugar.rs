use crate::builder;
use crate::env::Env;
use arena_alloc::General;
use ir::desugared::{
    Argument, Atom, Expr, Field, FieldDefinition, FunctionDefinition, Literal, Program, Statement,
    StructDefinition, Type,
};
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
        ir::ast::Definition::Struct { name, fields, .. } => {
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
        ir::ast::Expr::StructLiteral { name, fields, .. } => {
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
        ir::ast::Expr::Block(ir::ast::Block {
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

pub fn bind_pattern<'old, 'new, 'names, 'ident, 'meta>(
    to_bind: typed::Pattern<'old, 'ident>,
    bound_value: Atom,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'ident, 'meta>,
    alloc: &General<'new>,
) {
    match to_bind {
        ir::ast::Pattern::Variable(name, _) => {
            block.with_statement(Statement {
                variable: name.identifier.tag,
                r#type: r#type(name.r#type, env, alloc),
                value: Expr::Atom(bound_value),
            });
        }
        ir::ast::Pattern::Struct { fields, .. } => {
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

pub fn statement<'old, 'new, 'names, 'ident, 'meta>(
    to_desugar: typed::Statement<'old, 'ident>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'ident, 'meta>,
    alloc: &General<'new>,
) {
    match to_desugar {
        ir::ast::Statement::Let {
            left_side,
            right_side,
            ..
        } => {
            let right_side_result = expr(right_side, block, env, alloc);
            bind_pattern(left_side, right_side_result, block, env, alloc);
        }
        ir::ast::Statement::Raw(raw, _) => {
            expr(raw, block, env, alloc);
        }
    }
}

pub fn field_literal<'old, 'new, 'names, 'ident, 'meta>(
    to_desugar: typed::Field<'old, 'ident>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'new, 'ident, 'meta>,
    alloc: &General<'new>,
) -> Field {
    let name = to_desugar.name.identifier.tag;
    let value = expr(to_desugar.value, block, env, alloc);
    Field { name, value }
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
