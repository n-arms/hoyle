use crate::check::{field, pattern, pattern_field};
use crate::env::Env;
use crate::error::*;
use crate::unify::struct_type;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Literal, Span};
use ir::qualified;
use ir::typed::*;

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

            let return_type = if let Some(return_type) = return_type {
                Some(r#type(return_type, general))
            } else {
                None
            };

            let typed_body = expr(body, &mut inner_env, interner, general)?;

            Ok(Definition::Function {
                name,
                generics: general.alloc_slice_fill_iter(generics.iter().copied()),
                arguments: typed_arguments,
                return_type,
                body: typed_body,
                span,
            })
        }
        ir::ast::Definition::Struct { name, fields, span } => {
            let typed_fields = general.alloc_slice_try_fill_iter(
                fields
                    .into_iter()
                    .map(|field| field_definition(*field, general)),
            )?;
            env.bind_struct(
                UntypedIdentifier {
                    source: qualified::IdentifierSource::Local,
                    name,
                },
                typed_fields,
            );

            Ok(Definition::Struct {
                name,
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
                typed_right_side.r#type(interner, general),
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
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Literal<'new>> {
    match to_infer {
        Literal::Integer(integer) => Ok(Literal::Integer(general.alloc_str(integer))),
    }
}

pub fn branch<'old, 'new, 'ident>(
    to_infer: qualified::Branch<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Branch<'new, 'ident>> {
    todo!()
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
                    .into_iter()
                    .map(|arg| expr(*arg, env, interner, general)),
            )?;
            Ok(Expr::Call {
                function: general.alloc(typed_function),
                arguments: typed_arguments,
                span,
            })
        }
        ir::ast::Expr::Operation {
            operator,
            arguments,
            span,
        } => todo!(),
        ir::ast::Expr::Block(untyped_block) => {
            let typed_block = block(untyped_block, &mut env.clone(), interner, general)?;

            Ok(Expr::Block(typed_block))
        }
        ir::ast::Expr::Annotated {
            expr,
            annotation,
            span,
        } => todo!(),
        ir::ast::Expr::Case {
            predicate,
            branches,
            span,
        } => todo!(),
        ir::ast::Expr::StructLiteral { name, fields, span } => {
            let defined_type = env.lookup_struct(name);

            let typed_name = Identifier {
                source: name.source,
                name: name.name,
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
