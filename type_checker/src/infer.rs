use crate::check::{branch, field, pattern};
use crate::env::Env;
use crate::error::Result;
use crate::extract::{struct_type, Typeable};
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Literal, Span};
use ir::qualified;
use ir::typed::{
    Argument, Block, Definition, Expr, FieldDefinition, Identifier, Program, Statement, Type,
};

pub fn program<'old, 'new, 'ident>(
    to_infer: qualified::Program<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Program<'new, 'ident>> {
    let typed_definitions = general.alloc_slice_try_fill_iter(
        to_infer
            .definitions
            .iter()
            .map(|def| definition(*def, env, interner, general)),
    )?;

    Ok(Program {
        definitions: typed_definitions,
    })
}

pub fn field_definition<'old, 'new, 'ident>(
    to_infer: qualified::FieldDefinition<'old, 'ident>,
    general: &General<'new>,
) -> Result<'new, 'ident, FieldDefinition<'new, 'ident>> {
    let typed_field_type = r#type(to_infer.field_type, general);

    Ok(FieldDefinition {
        name: to_infer.name,
        field_type: typed_field_type,
        span: to_infer.span,
    })
}

fn arrow_type<'expr, 'ident, I>(
    arguments: I,
    r#return: Type<'expr, 'ident>,
    general: &General<'expr>,
) -> Type<'expr, 'ident>
where
    I: IntoIterator<Item = Type<'expr, 'ident>>,
    I::IntoIter: ExactSizeIterator,
{
    Type::Arrow {
        arguments: general.alloc_slice_fill_iter(arguments),
        return_type: general.alloc(r#return),
        span: None,
    }
}

pub fn definition<'old, 'new, 'ident>(
    to_infer: qualified::Definition<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Definition<'new, 'ident>> {
    match to_infer {
        ir::ast::Definition::Function {
            name,
            generics,
            arguments,
            return_type,
            body,
            span,
        } => {
            let mut inner_env = env.clone();
            let typed_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| argument(*arg, &mut inner_env, interner, general)),
            )?;

            let return_type = return_type.map(|return_type| r#type(return_type, general));

            let typed_body;

            if let Some(return_type) = return_type {
                let func_type = Type::Arrow {
                    arguments: general.alloc_slice_fill_iter(
                        typed_arguments.iter().map(|arg| arg.type_annotation),
                    ),
                    return_type: general.alloc(return_type),
                    span: None,
                };
                env.bind_variable(name, func_type);
                typed_body = expr(body, &mut inner_env, interner, general)?;
            } else {
                typed_body = expr(body, &mut inner_env, interner, general)?;
                env.bind_variable(name, typed_body.extract(&env.primitives));
            }

            let typed_name = Identifier::new(
                name,
                arrow_type(
                    typed_arguments.iter().map(|arg| arg.type_annotation),
                    typed_body.extract(&env.primitives),
                    general,
                ),
            );

            Ok(Definition::Function {
                name: typed_name,
                generics: general.alloc_slice_fill_iter(generics.iter().copied()),
                arguments: typed_arguments,
                return_type,
                body: typed_body,
                span,
            })
        }
        ir::ast::Definition::Struct { name, fields, span } => {
            let typed_fields = general.alloc_slice_try_fill_iter(
                fields.iter().map(|field| field_definition(*field, general)),
            )?;
            let typed_name = env.bind_struct(name, typed_fields);

            Ok(Definition::Struct {
                name: typed_name,
                fields: typed_fields,
                span,
            })
        }
    }
}

pub fn argument<'old, 'new, 'ident>(
    to_infer: qualified::Argument<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Argument<'new, 'ident>> {
    let type_annotation = r#type(to_infer.type_annotation, general);
    let typed_pattern = pattern(to_infer.pattern, type_annotation, env, interner, general)?;

    Ok(Argument {
        pattern: typed_pattern,
        type_annotation,
        span: to_infer.span,
    })
}

pub fn block<'old, 'new, 'ident>(
    to_infer: qualified::Block<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Block<'new, 'ident>> {
    let typed_statements = general.alloc_slice_try_fill_iter(
        to_infer
            .statements
            .iter()
            .map(|stmt| statement(*stmt, env, interner, general)),
    )?;

    let typed_result = if let Some(result) = to_infer.result {
        let alloc_result: &_ = general.alloc(expr(*result, env, interner, general)?);
        Some(alloc_result)
    } else {
        None
    };

    Ok(Block {
        statements: typed_statements,
        result: typed_result,
        span: to_infer.span,
    })
}

pub fn statement<'old, 'new, 'ident>(
    to_infer: qualified::Statement<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Statement<'new, 'ident>> {
    match to_infer {
        ir::ast::Statement::Let {
            left_side,
            right_side,
            span,
        } => {
            let typed_right_side = expr(right_side, env, interner, general)?;

            let typed_left_side = pattern(
                left_side,
                typed_right_side.extract(&env.primitives),
                env,
                interner,
                general,
            )?;

            Ok(Statement::Let {
                left_side: typed_left_side,
                right_side: typed_right_side,
                span,
            })
        }
        ir::ast::Statement::Raw(raw, span) => {
            let typed_raw = expr(raw, env, interner, general)?;

            Ok(Statement::Raw(typed_raw, span))
        }
    }
}
#[must_use]
pub fn r#type<'old, 'new, 'ident>(
    to_infer: qualified::Type<'old, 'ident, Span>,
    general: &General<'new>,
) -> Type<'new, 'ident> {
    match to_infer {
        qualified::Type::Named { name, span } => Type::Named {
            name,
            span: Some(span),
        },
        qualified::Type::Arrow {
            arguments,
            return_type,
            span,
        } => Type::Arrow {
            arguments: general
                .alloc_slice_fill_iter(arguments.iter().map(|arg| r#type(*arg, general))),
            return_type: general.alloc(r#type(*return_type, general)),
            span: Some(span),
        },
    }
}

pub fn literal<'old, 'new, 'ident>(
    to_infer: Literal<'old>,
    _interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Literal<'new>> {
    match to_infer {
        Literal::Integer(integer) => Ok(Literal::Integer(general.alloc_str(integer))),
    }
}

pub fn expr<'old, 'new, 'ident>(
    to_infer: qualified::Expr<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Expr<'new, 'ident>> {
    match to_infer {
        ir::ast::Expr::Variable(variable, span) => {
            let typed_variable = env.lookup_variable(variable);

            Ok(Expr::Variable(typed_variable, span))
        }
        ir::ast::Expr::Literal(lit, span) => {
            let typed_literal = literal(lit, interner, general)?;
            Ok(Expr::Literal(typed_literal, span))
        }
        ir::ast::Expr::Call {
            function,
            arguments,
            span,
        } => {
            let typed_function = expr(*function, env, interner, general)?;
            let typed_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| expr(*arg, env, interner, general)),
            )?;
            Ok(Expr::Call {
                function: general.alloc(typed_function),
                arguments: typed_arguments,
                span,
            })
        }
        ir::ast::Expr::Operation {
            operator: _,
            arguments: _,
            span: _,
        } => todo!(),
        ir::ast::Expr::Block(untyped_block) => {
            let typed_block = block(untyped_block, &mut env.clone(), interner, general)?;

            Ok(Expr::Block(typed_block))
        }
        ir::ast::Expr::Annotated {
            expr: _,
            annotation: _,
            span: _,
        } => todo!(),
        ir::ast::Expr::Case {
            predicate,
            branches,
            span,
        } => {
            let typed_predicate = expr(*predicate, env, interner, general)?;

            let typed_branches = general.alloc_slice_try_fill_iter(branches.iter().map(|b| {
                branch(
                    *b,
                    typed_predicate.extract(&env.primitives),
                    env,
                    interner,
                    general,
                )
            }))?;

            Ok(Expr::Case {
                predicate: general.alloc(typed_predicate),
                branches: typed_branches,
                span,
            })
        }
        ir::ast::Expr::StructLiteral { name, fields, span } => {
            let defined_type = env.lookup_struct(name);

            let typed_name = Identifier {
                identifier: name,
                r#type: struct_type(name),
            };

            let typed_fields = general.alloc_slice_try_fill_iter(
                fields
                    .iter()
                    .map(|to_check| field(*to_check, defined_type, env, interner, general)),
            )?;

            Ok(Expr::StructLiteral {
                name: typed_name,
                fields: typed_fields,
                span,
            })
        }
    }
}
