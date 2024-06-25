use arena_alloc::General;
use ir::{qualified, typed};

use crate::env::Env;
use crate::error::{Error, Result};

pub fn pattern<'old, 'new>(
    to_check: &qualified::Pattern<'old>,
    against: &typed::Type<'new>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Pattern<'new>> {
    match to_check {
        qualified::Pattern::Variable { name, span } => Ok(typed::Pattern::Variable {
            name: name.clone(),
            r#type: against.clone(),
        }),
        qualified::Pattern::Struct { name, fields, span } => {
            let field_types: &[typed::FieldDefinition] = match against {
                typed::Type::Named { name, arguments } => env.lookup_struct(name).fields,
                _ => panic!("expected struct"),
            };
            let zipped_fields = field_types.iter().map(|field_def| {
                (
                    field_def.field.clone(),
                    field_def.r#type.clone(),
                    fields
                        .iter()
                        .find_map(|field| {
                            if field.name == field_def.field {
                                Some(field.pattern.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap(),
                )
            });
            let typed_fields =
                general.alloc_slice_try_fill_iter(zipped_fields.map(|(name, r#type, p)| {
                    Ok(typed::PatternField {
                        name,
                        pattern: pattern(&p, &r#type, env, general)?,
                    })
                }))?;
            Ok(typed::Pattern::Struct {
                name: name.clone(),
                fields: typed_fields,
                r#type: against.clone(),
            })
        }
    }
}

/*
use crate::env::Env;
use crate::error::Result;
use crate::extract::{struct_type, Typeable};
use crate::infer;
use crate::substitute::Substitution;
use crate::unify;
use arena_alloc::{General, Interning, Specialized};

use ir::qualified::{self, Type};
use ir::typed::{Branch, Expr, Field, FieldDefinition, Identifier, Pattern, PatternField};

pub fn expr<'old, 'new, 'names>(
    to_check: qualified::Expr<'old>,
    target: Type<'new>,
    env: &mut Env<'new, 'names>,
    substitution: &mut Substitution<'new>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Expr<'new>> {
    let typed_expr = infer::expr(to_check, env, substitution, interner, general)?;

    unify::check_types(target, typed_expr.extract(&env.primitives))?;

    Ok(typed_expr)
}

pub fn field<'old, 'new, 'names>(
    to_check: qualified::Field<'old>,
    target_fields: &'new [FieldDefinition<'new>],
    env: &mut Env<'new, 'names>,
    substitution: &mut Substitution<'new>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Field<'new>> {
    let target_field = target_fields
        .iter()
        .find(|field| field.name.identifier == to_check.name)
        .expect("qualifier should have caught undefined field");

    let typed_value = expr(
        to_check.value,
        target_field.field_type,
        env,
        substitution,
        interner,
        general,
    )?;

    Ok(Field {
        name: target_field.name,
        value: typed_value,
        span: to_check.span,
    })
}

pub fn pattern_field<'old, 'new, 'names>(
    to_check: qualified::PatternField<'old>,
    target_fields: &'new [FieldDefinition<'new>],
    env: &mut Env<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, PatternField<'new>> {
    let target_field = target_fields
        .iter()
        .find(|field| field.name.identifier == to_check.name)
        .expect("qualification should have caught undefined fields");

    let typed_pattern = pattern(
        to_check.pattern,
        target_field.field_type,
        env,
        interner,
        general,
    )?;

    Ok(PatternField {
        name: target_field.name,
        pattern: typed_pattern,
        span: to_check.span,
    })
}

pub fn pattern<'old, 'new, 'names>(
    to_check: qualified::Pattern<'old>,
    target: Type<'new>,
    env: &mut Env<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Pattern<'new>> {
    match to_check {
        ir::ast::Pattern::Variable(variable, span) => {
            env.bind_unqualified_variable(variable, target);

            Ok(Pattern::Variable(Identifier::new(variable, target), span))
        }
        ir::ast::Pattern::Struct { name, fields, span } => {
            let to_check_type = struct_type(name);
            unify::check_types(to_check_type, target)?;

            let struct_definition = env.lookup_struct(name);
            let typed_fields =
                general.alloc_slice_try_fill_iter(fields.iter().map(|field| {
                    pattern_field(*field, struct_definition, env, interner, general)
                }))?;

            Ok(Pattern::Struct {
                name: Identifier {
                    identifier: name,
                    r#type: to_check_type,
                },
                fields: typed_fields,
                span,
            })
        }
    }
}

pub fn branch<'old, 'new, 'names>(
    to_check: qualified::Branch<'old>,
    target_pattern_type: Type<'new>,
    env: &mut Env<'new, 'names>,
    substitution: &mut Substitution<'new>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Branch<'new>> {
    let typed_pattern = pattern(
        to_check.pattern,
        target_pattern_type,
        env,
        interner,
        general,
    )?;

    let typed_body = infer::expr(to_check.body, env, substitution, interner, general)?;

    Ok(Branch {
        pattern: typed_pattern,
        body: typed_body,
        span: to_check.span,
    })
}
*/
