use crate::definitions::Definitions;
use crate::error::Result;
use arena_alloc::{General, Interning, Specialized};
use ir::ast;
use ir::qualified::{
    Argument, Block, Definition, Expr, Field, Identifier, IdentifierSource, Path, Pattern, Program,
    Statement, Type, TypeField, TypeName,
};

pub fn program<'old, 'new, 'ident>(
    to_qualify: ast::Program<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Program<'new, 'ident>> {
    let qualified_definitions = general.alloc_slice_try_fill_iter(
        to_qualify
            .definitions
            .iter()
            .map(|def| definition(*def, definitions, interner, general)),
    )?;

    Ok(Program {
        definitions: qualified_definitions,
    })
}

pub fn definition<'old, 'new, 'ident>(
    to_qualify: ast::Definition<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Definition<'new, 'ident>> {
    let identifier = Identifier {
        source: IdentifierSource::Global(Path::Current),
        name: to_qualify.name,
        r#type: Type::Wildcard,
    };
    definitions.with_variables([(to_qualify.name, identifier)]);

    let mut inner_defs = definitions.clone();
    inner_defs.with_types(to_qualify.generics.iter().map(|generic| {
        (
            generic.identifier,
            TypeName {
                source: IdentifierSource::Local,
                name: generic.identifier,
            },
        )
    }));

    let return_type = to_qualify
        .return_type
        .map_or(Ok(Type::Wildcard), |result| {
            r#type(result, &mut inner_defs, interner, general)
        })?;

    let arguments = general.alloc_slice_try_fill_iter(
        to_qualify
            .arguments
            .iter()
            .map(|arg| argument(*arg, &mut inner_defs, interner, general)),
    )?;

    let body = expr(to_qualify.body, &mut inner_defs, interner, general)?;

    let generics = general.alloc_slice_fill_iter(to_qualify.generics.iter().copied());

    Ok(Definition {
        name: to_qualify.name,
        generics,
        arguments,
        return_type: Some(return_type),
        body,
        span: to_qualify.span,
    })
}

pub fn argument<'old, 'new, 'ident>(
    to_qualify: ast::Argument<'old, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Argument<'new, 'ident>> {
    let qualified_pattern = pattern(to_qualify.pattern, definitions, interner, general)?;
    let qualified_type = r#type(to_qualify.type_annotation, definitions, interner, general)?;

    Ok(Argument {
        pattern: qualified_pattern,
        type_annotation: qualified_type,
        span: to_qualify.span,
    })
}

pub fn statement<'old, 'new, 'ident>(
    to_qualify: ast::Statement<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Statement<'new, 'ident>> {
    match to_qualify {
        ast::Statement::Let {
            left_side,
            right_side,
            span,
        } => {
            let qualified_left_side = pattern(left_side, definitions, interner, general)?;
            let qualified_right_side = expr(right_side, definitions, interner, general)?;

            Ok(Statement::Let {
                left_side: qualified_left_side,
                right_side: qualified_right_side,
                span,
            })
        }
        ast::Statement::Raw(raw_expr, span) => {
            let qualified_raw_expr = expr(raw_expr, definitions, interner, general)?;

            Ok(Statement::Raw(qualified_raw_expr, span))
        }
    }
}

pub fn pattern<'old, 'new, 'ident>(
    to_qualify: ast::Pattern<'old, &'ident str>,
    definitions: &mut Definitions<'new, 'ident>,
    _interner: &Interning<'ident, Specialized>,
    _general: &General<'new>,
) -> Result<'ident, Pattern<'new, 'ident>> {
    match to_qualify {
        ast::Pattern::Variable(variable, span) => {
            let qualified = Identifier {
                source: IdentifierSource::Local,
                name: variable,
                r#type: Type::Wildcard,
            };
            definitions.with_variables([(variable, qualified)]);
            Ok(Pattern::Variable(qualified, span))
        }
        ast::Pattern::Tuple(_, _) => todo!(),
    }
}

pub fn block<'old, 'new, 'ident>(
    to_qualify: ast::Block<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Block<'new, 'ident>> {
    let statements = general.alloc_slice_try_fill_iter(
        to_qualify
            .statements
            .iter()
            .map(|stmt| statement(*stmt, definitions, interner, general)),
    )?;
    let result = if let Some(result) = to_qualify.result {
        let boxed_result: &_ = general.alloc(expr(*result, definitions, interner, general)?);
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

pub fn type_field<'old, 'new, 'ident>(
    to_qualify: ast::TypeField<'old, 'ident>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, TypeField<'new, 'ident>> {
    let qualified_field_type = r#type(to_qualify.field_type, definitions, interner, general)?;

    Ok(TypeField {
        name: to_qualify.name,
        field_type: qualified_field_type,
        span: to_qualify.span,
    })
}

pub fn r#type<'old, 'new, 'ident>(
    to_qualify: ast::Type<'old, 'ident>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Type<'new, 'ident>> {
    match to_qualify {
        ast::Type::Named(type_name, span) => {
            let qualified_type_name = definitions.lookup_type(type_name)?;

            Ok(Type::Named {
                name: qualified_type_name,
                span,
            })
        }
        ast::Type::Variant {
            tag,
            arguments,
            span,
        } => {
            let qualified_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| r#type(*arg, definitions, interner, general)),
            )?;

            Ok(Type::Variant {
                tag,
                arguments: qualified_arguments,
                span,
            })
        }
        ast::Type::Record { fields, span } => {
            let qualified_fields = general.alloc_slice_try_fill_iter(
                fields
                    .iter()
                    .map(|f| type_field(*f, definitions, interner, general)),
            )?;

            Ok(Type::Record {
                fields: qualified_fields,
                span,
            })
        }
        ast::Type::Arrow {
            arguments,
            return_type,
            span,
        } => {
            let qualified_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| r#type(*arg, definitions, interner, general)),
            )?;

            let qualified_return_type =
                general.alloc(r#type(*return_type, definitions, interner, general)?);

            Ok(Type::Arrow {
                arguments: qualified_arguments,
                return_type: qualified_return_type,
                span,
            })
        }
        ast::Type::Union { cases, span } => {
            let qualified_cases = general.alloc_slice_try_fill_iter(
                cases
                    .iter()
                    .map(|case| r#type(*case, definitions, interner, general)),
            )?;

            Ok(Type::Union {
                cases: qualified_cases,
                span,
            })
        }
    }
}

pub fn literal<'old, 'new, 'ident>(
    to_qualify: ast::Literal<'old>,
    _definitions: &mut Definitions<'new, 'ident>,
    _interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, ast::Literal<'new>> {
    match to_qualify {
        ast::Literal::Integer(int) => Ok(ast::Literal::Integer(general.alloc_str(int))),
    }
}

pub fn field<'old, 'new, 'ident>(
    to_qualify: ast::Field<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Field<'new, 'ident>> {
    let qualified_value = expr(to_qualify.value, definitions, interner, general)?;

    Ok(Field {
        name: to_qualify.name,
        value: qualified_value,
        span: to_qualify.span,
    })
}

pub fn expr<'old, 'new, 'ident>(
    to_qualify: ast::Expr<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Expr<'new, 'ident>> {
    match to_qualify {
        ast::Expr::Variable(variable, span) => {
            let qualified_variable = definitions.lookup_variable(variable)?;
            Ok(Expr::Variable(qualified_variable, span))
        }
        ast::Expr::Literal(lit, span) => {
            let qualified_lit = literal(lit, definitions, interner, general)?;
            Ok(Expr::Literal(qualified_lit, span))
        }
        ast::Expr::Call {
            function,
            arguments,
            span,
        } => {
            let qualified_function =
                general.alloc(expr(*function, definitions, interner, general)?);
            let qualified_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| expr(*arg, definitions, interner, general)),
            )?;

            Ok(Expr::Call {
                function: qualified_function,
                arguments: qualified_arguments,
                span,
            })
        }
        ast::Expr::Operation { .. } => todo!(),
        ast::Expr::Record { fields, span } => {
            let qualified_fields = general.alloc_slice_try_fill_iter(
                fields
                    .iter()
                    .map(|f| field(*f, definitions, interner, general)),
            )?;
            Ok(Expr::Record {
                fields: qualified_fields,
                span,
            })
        }
        ast::Expr::Block(statements) => {
            let qualified_block = block(statements, definitions, interner, general)?;
            Ok(Expr::Block(qualified_block))
        }
        ast::Expr::Annotated { .. } => todo!(),
        ast::Expr::Variant {
            variant,
            arguments,
            span,
        } => {
            let qualified_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| expr(*arg, definitions, interner, general)),
            )?;
            Ok(Expr::Variant {
                variant,
                arguments: qualified_arguments,
                span,
            })
        }
    }
}
