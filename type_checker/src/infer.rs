use crate::env::Env;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Literal, Span};
use ir::qualified;
use ir::typed::*;
use std::result;

#[derive(Debug)]
pub enum Error {}

type Result<T> = result::Result<T, Error>;

pub fn program<'old, 'new, 'ident>(
    to_infer: qualified::Program<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Program<'new, 'ident>> {
    let typed_definitions = general.alloc_slice_try_fill_iter(
        to_infer
            .definitions
            .iter()
            .map(|def| definition(*def, &mut (env.clone()), interner, general)),
    )?;

    Ok(Program {
        definitions: typed_definitions,
    })
}

pub fn definition<'old, 'new, 'ident>(
    to_infer: qualified::Definition<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Definition<'new, 'ident>> {
    let typed_arguments = general.alloc_slice_try_fill_iter(
        to_infer
            .arguments
            .iter()
            .map(|arg| argument(*arg, env, interner, general)),
    )?;

    let return_type = if let Some(return_type) = to_infer.return_type {
        Some(r#type(return_type, general))
    } else {
        None
    };

    let typed_body = expr(to_infer.body, env, interner, general)?;

    Ok(Definition {
        name: to_infer.name,
        generics: general.alloc_slice_fill_iter(to_infer.generics.iter().copied()),
        arguments: typed_arguments,
        return_type,
        body: typed_body,
        span: to_infer.span,
    })
}

pub fn argument<'old, 'new, 'ident>(
    to_infer: qualified::Argument<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Argument<'new, 'ident>> {
    let type_annotation = r#type(to_infer.type_annotation, general);
    let typed_pattern = check_pattern(to_infer.pattern, type_annotation, env, interner, general)?;

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
) -> Result<Block<'new, 'ident>> {
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
) -> Result<Statement<'new, 'ident>> {
    match to_infer {
        ir::ast::Statement::Let {
            left_side,
            right_side,
            span,
        } => {
            let typed_right_side = expr(right_side, env, interner, general)?;

            let typed_left_side = check_pattern(
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

pub fn check_pattern_field<'old, 'new, 'ident>(
    to_check: qualified::PatternField<'old, 'ident>,
    checking_type: Type<'new, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<PatternField<'new, 'ident>> {
    todo!()
}

pub fn check_pattern<'old, 'new, 'ident>(
    to_check: qualified::Pattern<'old, 'ident>,
    checking_type: Type<'new, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Pattern<'new, 'ident>> {
    match to_check {
        ir::ast::Pattern::Variable(variable, span) => {
            env.bind_variable(variable, checking_type);

            Ok(Pattern::Variable(
                Identifier::new(variable, checking_type),
                span,
            ))
        }
        ir::ast::Pattern::Variant {
            tag,
            arguments,
            span,
        } => todo!(),
        ir::ast::Pattern::Record { fields, span } => todo!(),
    }
}

pub fn type_field<'old, 'new, 'ident>(
    to_infer: qualified::TypeField<'old, 'ident, Span>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<qualified::TypeField<'new, 'ident, Span>> {
    todo!()
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
        qualified::Type::Variant {
            tag,
            arguments,
            span,
        } => Type::Variant {
            tag,
            arguments: general
                .alloc_slice_fill_iter(arguments.iter().map(|arg| r#type(*arg, general))),
            span: Some(span),
        },
        qualified::Type::Record { fields, span } => todo!(),
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
        qualified::Type::Union { cases, span } => todo!(),
    }
}

pub fn literal<'old, 'new, 'ident>(
    to_infer: Literal<'old>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Literal<'new>> {
    match to_infer {
        Literal::Integer(integer) => Ok(Literal::Integer(general.alloc_str(integer))),
    }
}

pub fn field<'old, 'new, 'ident>(
    to_infer: qualified::Field<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Field<'new, 'ident>> {
    todo!()
}

pub fn branch<'old, 'new, 'ident>(
    to_infer: qualified::Branch<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Branch<'new, 'ident>> {
    todo!()
}

pub fn expr<'old, 'new, 'ident>(
    to_infer: qualified::Expr<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<Expr<'new, 'ident>> {
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
        } => todo!(),
        ir::ast::Expr::Operation {
            operator,
            arguments,
            span,
        } => todo!(),
        ir::ast::Expr::Variant {
            tag,
            arguments,
            span,
        } => {
            let typed_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| expr(*arg, env, interner, general)),
            )?;

            Ok(Expr::Variant {
                tag,
                arguments: typed_arguments,
                span,
            })
        }
        ir::ast::Expr::Record { fields, span } => todo!(),
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
    }
}
