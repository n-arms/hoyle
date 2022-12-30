use crate::env::Env;
use crate::error::*;
use crate::unify::{struct_type, unify_types};
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Literal, Span};
use ir::qualified;
use ir::typed::*;

pub fn expr<'old, 'new, 'ident>(
    to_check: qualified::Expr<'old, 'ident>,
    target: Type<'new, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Expr<'new, 'ident>> {
    match to_check {
        ir::ast::Expr::Variable(identifier, span) => {
            env.check_variable(identifier, target)?;
            todo!()
        }
        ir::ast::Expr::Literal(literal, span) => Ok(Expr::Literal(literal.realloc(general), span)),
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
        ir::ast::Expr::StructLiteral { name, fields, span } => todo!(),
        ir::ast::Expr::Block(_) => todo!(),
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

pub fn field<'old, 'new, 'ident>(
    to_check: qualified::Field<'old, 'ident>,
    target_fields: &'new [FieldDefinition<'new, 'ident>],
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Field<'new, 'ident>> {
    let target_field = target_fields
        .iter()
        .find(|field| field.name == to_check.name)
        .expect("qualifier should have caught undefined field");

    let typed_value = expr(
        to_check.value,
        target_field.field_type,
        env,
        interner,
        general,
    )?;

    Ok(Field {
        name: to_check.name,
        value: typed_value,
        span: to_check.span,
    })
}

pub fn pattern_field<'old, 'new, 'ident>(
    to_check: qualified::PatternField<'old, 'ident>,
    target_fields: &'new [FieldDefinition<'new, 'ident>],
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, PatternField<'new, 'ident>> {
    let target_field = target_fields
        .iter()
        .find(|field| field.name == to_check.name)
        .expect("qualification should have caught undefined fields");

    let typed_pattern = pattern(
        to_check.pattern,
        target_field.field_type,
        env,
        interner,
        general,
    )?;

    Ok(PatternField {
        name: to_check.name,
        pattern: typed_pattern,
        span: to_check.span,
    })
}

pub fn pattern<'old, 'new, 'ident>(
    to_check: qualified::Pattern<'old, 'ident>,
    target: Type<'new, 'ident>,
    env: &mut Env<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Pattern<'new, 'ident>> {
    match to_check {
        ir::ast::Pattern::Variable(variable, span) => {
            env.bind_variable(variable, target);

            Ok(Pattern::Variable(Identifier::new(variable, target), span))
        }
        ir::ast::Pattern::Struct { name, fields, span } => {
            let to_check_type = struct_type(name);
            unify_types(to_check_type, target)?;

            let struct_definition = env.lookup_struct(name);
            let typed_fields =
                general.alloc_slice_try_fill_iter(fields.iter().map(|field| {
                    pattern_field(*field, struct_definition, env, interner, general)
                }))?;

            Ok(Pattern::Struct {
                name: Identifier {
                    source: name.source,
                    name: name.name,
                    r#type: to_check_type,
                },
                fields: typed_fields,
                span,
            })
        }
    }
}