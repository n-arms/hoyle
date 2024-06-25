use crate::definitions::Local;
use crate::error::{Error, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::qualified::{
    ArgumentDefinition, Block, Branch, Definition, Expr, Field, FieldDefinition,
    FunctionDefinition, GenericDefinition, Literal, Pattern, PatternField, Program, Statement,
    StructDefinition, Type,
};
use ir::source;

pub fn program<'old, 'new, 'names>(
    to_qualify: source::Program<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Program<'new>> {
    let qualified_definitions = general.alloc_slice_try_fill_iter(
        to_qualify
            .definitions
            .iter()
            .map(|def| definition(def.clone(), definitions, interner, general)),
    )?;

    Ok(Program {
        definitions: qualified_definitions,
    })
}

pub fn definition<'old, 'new, 'names>(
    to_qualify: source::Definition<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Definition<'new>> {
    match to_qualify {
        source::Definition::Function(source::FunctionDefinition {
            name,
            generics,
            arguments,
            return_type,
            body,
            span,
        }) => {
            let identifier = definitions.define_local_variable(name);

            let mut inner_defs = definitions.clone();

            let generics = general.alloc_slice_try_fill_iter(
                generics.iter().map(|g| generic(g.clone(), &mut inner_defs)),
            )?;

            let return_type = r#type(return_type, &mut inner_defs, general)?;

            let arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| argument(arg.clone(), &mut inner_defs, interner, general)),
            )?;

            let body = expr(body, &mut inner_defs, interner, general)?;

            Ok(Definition::Function(FunctionDefinition {
                name: identifier,
                generics,
                arguments,
                return_type,
                body,
                span,
            }))
        }
        source::Definition::Struct(source::StructDefinition { name, fields, span }) => {
            let qualified_fields = general.alloc_slice_try_fill_iter(
                fields
                    .iter()
                    .map(|field| field_definition(field.clone(), definitions, general)),
            )?;

            let def = definitions.define_local_struct(name, span, qualified_fields);

            Ok(Definition::Struct(StructDefinition {
                name: def.name,
                fields: qualified_fields,
                span,
            }))
        }
    }
}

pub fn generic<'old, 'new, 'names>(
    to_qualify: source::GenericDefinition,
    definitions: &mut Local<'new, 'names>,
) -> Result<'old, 'new, GenericDefinition> {
    let name = definitions.define_local_type(to_qualify.name);

    Ok(GenericDefinition { name })
}

pub fn field_definition<'old, 'new, 'names>(
    to_qualify: source::FieldDefinition<'old>,
    definitions: &mut Local<'new, 'names>,
    general: &General<'new>,
) -> Result<'old, 'new, FieldDefinition<'new>> {
    let qualified_field_type = r#type(to_qualify.r#type, definitions, general)?;
    let qualified_field_name = definitions.define_local_field(to_qualify.name);

    Ok(FieldDefinition {
        name: qualified_field_name,
        r#type: qualified_field_type,
        span: to_qualify.span,
    })
}

pub fn argument<'old, 'new, 'names>(
    to_qualify: source::ArgumentDefinition<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, ArgumentDefinition<'new>> {
    let qualified_pattern = pattern(to_qualify.pattern, definitions, interner, general)?;
    let qualified_type = r#type(to_qualify.r#type, definitions, general)?;

    Ok(ArgumentDefinition {
        pattern: qualified_pattern,
        r#type: qualified_type,
        span: to_qualify.span,
    })
}

pub fn statement<'old, 'new, 'names>(
    to_qualify: source::Statement<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Statement<'new>> {
    match to_qualify {
        source::Statement::Let {
            pattern: let_pattern,
            value: let_value,
            span,
        } => {
            let qualified_pattern = pattern(let_pattern, definitions, interner, general)?;
            let qualified_value = expr(let_value, definitions, interner, general)?;

            Ok(Statement::Let {
                pattern: qualified_pattern,
                value: qualified_value,
                span,
            })
        }
        source::Statement::Raw(raw_expr) => {
            let qualified_raw_expr = expr(raw_expr, definitions, interner, general)?;

            Ok(Statement::Raw(qualified_raw_expr))
        }
    }
}

pub fn pattern_field<'old, 'new, 'names>(
    to_qualify: source::PatternField<'old>,
    target_fields: &'new [FieldDefinition<'new>],
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, PatternField<'new>> {
    let span = to_qualify.span;
    let target_field = target_fields
        .iter()
        .find(|field| field.name.name == to_qualify.name.name)
        .ok_or(Error::StructPatternMissingField(
            to_qualify.clone(),
            target_fields,
        ))?;

    let qualified_value = pattern(to_qualify.pattern, definitions, interner, general)?;

    Ok(PatternField {
        name: target_field.name.clone(),
        pattern: qualified_value,
        span,
    })
}

pub fn pattern<'old, 'new, 'names>(
    to_qualify: source::Pattern<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Pattern<'new>> {
    match to_qualify {
        source::Pattern::Variable { name, span } => {
            let qualified = definitions.define_local_variable(name);
            Ok(Pattern::Variable {
                name: qualified,
                span,
            })
        }
        source::Pattern::Struct { name, fields, span } => {
            let struct_definition = definitions.lookup_struct(name)?;
            let qualified_fields = general.alloc_slice_try_fill_iter(fields.iter().map(|f| {
                pattern_field(
                    f.clone(),
                    struct_definition.fields,
                    definitions,
                    interner,
                    general,
                )
            }))?;
            Ok(Pattern::Struct {
                name: struct_definition.name,
                fields: qualified_fields,
                span,
            })
        }
    }
}

pub fn block<'old, 'new, 'names>(
    to_qualify: source::Block<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Block<'new>> {
    let statements = general.alloc_slice_try_fill_iter(
        to_qualify
            .statements
            .iter()
            .map(|stmt| statement(stmt.clone(), definitions, interner, general)),
    )?;
    let result = if let Some(result) = to_qualify.result {
        let boxed_result: &_ = general.alloc(expr(result.clone(), definitions, interner, general)?);
        Some(boxed_result)
    } else {
        None
    };
    Ok(Block {
        statements,
        result,
        span: to_qualify.span,
    })
}

pub fn r#type<'old, 'new, 'names>(
    to_qualify: source::Type<'old>,
    definitions: &mut Local<'new, 'names>,
    general: &General<'new>,
) -> Result<'old, 'new, Type<'new>> {
    match to_qualify {
        source::Type::Named { name, span } => {
            let qualified_type_name = definitions.lookup_type(name)?;

            Ok(Type::Named {
                name: qualified_type_name,
                span,
            })
        }
        source::Type::Function {
            arguments,
            return_type,
            span,
        } => {
            let qualified_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| r#type(arg.clone(), definitions, general)),
            )?;

            let qualified_return_type =
                general.alloc(r#type(return_type.clone(), definitions, general)?);

            Ok(Type::Function {
                arguments: qualified_arguments,
                return_type: qualified_return_type,
                span,
            })
        }
        source::Type::Application {
            main,
            arguments,
            span,
        } => {
            let qualified_main = definitions.lookup_type(main)?;

            let qualified_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| r#type(arg.clone(), definitions, general)),
            )?;

            Ok(Type::Application {
                main: qualified_main,
                arguments: qualified_arguments,
                span,
            })
        }
    }
}

pub fn field<'old, 'new, 'names>(
    to_qualify: source::Field<'old>,
    target_fields: &'new [FieldDefinition<'new>],
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Field<'new>> {
    let target_field = target_fields
        .iter()
        .find(|field| field.name.name == to_qualify.name.name)
        .ok_or(Error::StructLiteralContainsExtraField(
            to_qualify.clone(),
            target_fields,
        ))?;
    let qualified_value = expr(to_qualify.value, definitions, interner, general)?;

    Ok(Field {
        name: target_field.name.clone(),
        value: qualified_value,
    })
}

pub fn branch<'old, 'new, 'names>(
    to_qualify: source::Branch<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Branch<'new>> {
    let qualified_pattern = pattern(to_qualify.pattern, definitions, interner, general)?;
    let qualified_body = expr(to_qualify.body, definitions, interner, general)?;

    Ok(Branch {
        pattern: qualified_pattern,
        body: qualified_body,
        span: to_qualify.span,
    })
}

pub fn expr<'old, 'new, 'names>(
    to_qualify: source::Expr<'old>,
    definitions: &mut Local<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'old, 'new, Expr<'new>> {
    match to_qualify {
        source::Expr::Variable { identifier, span } => {
            let qualified_variable = definitions.lookup_variable(identifier)?;
            Ok(Expr::Variable {
                identifier: qualified_variable,
                span,
            })
        }
        source::Expr::Literal { literal, span } => {
            let qualified_literal = match literal {
                source::Literal::Boolean(boolean) => Literal::Boolean(boolean),
                source::Literal::Integer(integer) => Literal::Integer(integer),
            };
            Ok(Expr::Literal {
                literal: qualified_literal,
                span,
            })
        }
        source::Expr::Call {
            function,
            arguments,
            span,
        } => {
            let qualified_function =
                general.alloc(expr(function.clone(), definitions, interner, general)?);
            let qualified_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| expr(arg.clone(), definitions, interner, general)),
            )?;

            Ok(Expr::Call {
                function: qualified_function,
                arguments: qualified_arguments,
                span,
            })
        }
        source::Expr::Operation { .. } => todo!(),
        source::Expr::StructLiteral { name, fields, span } => {
            let definition = definitions.lookup_struct(name)?;
            let qualified_fields =
                general.alloc_slice_try_fill_iter(fields.iter().map(|f| {
                    field(f.clone(), definition.fields, definitions, interner, general)
                }))?;

            struct_contains_fields(qualified_fields, definition.fields)?;

            Ok(Expr::StructLiteral {
                name: definition.name,
                fields: qualified_fields,
                span,
            })
        }
        source::Expr::Block(statements) => {
            let qualified_block = block(statements, definitions, interner, general)?;
            Ok(Expr::Block(qualified_block))
        }
        source::Expr::Annotated { .. } => todo!(),
        source::Expr::Case {
            predicate,
            branches,
            span,
        } => {
            let qualified_predicate =
                general.alloc(expr(predicate.clone(), definitions, interner, general)?);

            let qualified_branches = general.alloc_slice_try_fill_iter(
                branches
                    .iter()
                    .map(|b| branch(b.clone(), definitions, interner, general)),
            )?;

            Ok(Expr::Case {
                predicate: qualified_predicate,
                branches: qualified_branches,
                span,
            })
        }
    }
}

pub fn struct_contains_fields<'old, 'new>(
    to_check: &'new [Field<'new>],
    must_have: &[FieldDefinition<'new>],
) -> Result<'old, 'new, ()> {
    for required_field in must_have {
        let missing_field = !to_check
            .iter()
            .any(|field| field.name.name == required_field.name.name);
        if missing_field {
            return Err(Error::StructLiteralMissingField(
                required_field.clone(),
                to_check,
            ));
        }
    }

    Ok(())
}
