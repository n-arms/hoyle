use crate::definitions::Definitions;
use crate::error::{Error, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::ast;
use ir::qualified::{
    Argument, Block, Branch, Definition, Expr, Field, FieldDefinition, Identifier,
    IdentifierSource, Path, Pattern, PatternField, Program, Statement, StructDefinition, Type,
    TypeField, TypeName,
};

pub fn program<'old, 'new, 'ident>(
    to_qualify: ast::Program<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Program<'new, 'ident>> {
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
) -> Result<'new, 'ident, Definition<'new, 'ident>> {
    match to_qualify {
        ast::Definition::Function {
            name,
            generics,
            arguments,
            return_type,
            body,
            span,
        } => {
            let identifier = Identifier {
                source: IdentifierSource::Global(Path::Current),
                name,
                r#type: None,
            };
            definitions.with_variables([(name, identifier)]);

            let mut inner_defs = definitions.clone();
            inner_defs.with_types(generics.iter().map(|generic| {
                (
                    generic.identifier,
                    TypeName {
                        source: IdentifierSource::Local,
                        name: generic.identifier,
                    },
                )
            }));

            let return_type = return_type.map_or(Ok(None), |result| {
                r#type(result, &mut inner_defs, interner, general).map(Some)
            })?;

            let arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| argument(*arg, &mut inner_defs, interner, general)),
            )?;

            let body = expr(body, &mut inner_defs, interner, general)?;

            let generics = general.alloc_slice_fill_iter(generics.iter().copied());

            Ok(Definition::Function {
                name,
                generics,
                arguments,
                return_type,
                body,
                span,
            })
        }
        ast::Definition::Struct { name, fields, span } => {
            let qualified_fields = general.alloc_slice_try_fill_iter(
                fields
                    .iter()
                    .map(|field| field_definition(*field, definitions, interner, general)),
            )?;

            definitions.with_struct(
                name,
                StructDefinition {
                    source: IdentifierSource::Local,
                    name,
                    fields: qualified_fields,
                },
            );

            Ok(Definition::Struct {
                name,
                fields: qualified_fields,
                span,
            })
        }
    }
}

pub fn field_definition<'old, 'new, 'ident>(
    to_qualify: ast::FieldDefinition<'ident, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, FieldDefinition<'new, 'ident>> {
    let qualified_field_type = r#type(to_qualify.field_type, definitions, interner, general)?;

    Ok(FieldDefinition {
        name: to_qualify.name,
        field_type: qualified_field_type,
        span: to_qualify.span,
    })
}

pub fn argument<'old, 'new, 'ident>(
    to_qualify: ast::Argument<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Argument<'new, 'ident>> {
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
) -> Result<'new, 'ident, Statement<'new, 'ident>> {
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

pub fn pattern_field<'old, 'new, 'ident>(
    to_qualify: ast::PatternField<'old, 'ident, &'ident str>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, PatternField<'new, 'ident>> {
    let qualified_value = pattern(to_qualify.pattern, definitions, interner, general)?;

    Ok(PatternField {
        name: to_qualify.name,
        pattern: qualified_value,
        span: to_qualify.span,
    })
}

pub fn pattern<'old, 'new, 'ident>(
    to_qualify: ast::Pattern<'old, 'ident, &'ident str>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Pattern<'new, 'ident>> {
    match to_qualify {
        ast::Pattern::Variable(variable, span) => {
            let qualified = Identifier {
                source: IdentifierSource::Local,
                name: variable,
                r#type: None,
            };
            definitions.with_variables([(variable, qualified)]);
            Ok(Pattern::Variable(qualified, span))
        }
        ast::Pattern::Struct { name, fields, span } => {
            let struct_definition = definitions.lookup_struct(name)?;
            let qualified_name = Identifier {
                name: struct_definition.name,
                source: struct_definition.source,
                r#type: None,
            };
            let qualified_fields = general.alloc_slice_try_fill_iter(
                fields
                    .iter()
                    .map(|f| pattern_field(*f, definitions, interner, general)),
            )?;
            Ok(Pattern::Struct {
                name: qualified_name,
                fields: qualified_fields,
                span,
            })
        }
    }
}

pub fn block<'old, 'new, 'ident>(
    to_qualify: ast::Block<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Block<'new, 'ident>> {
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
) -> Result<'new, 'ident, TypeField<'new, 'ident, ast::Span>> {
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
) -> Result<'new, 'ident, Type<'new, 'ident, ast::Span>> {
    match to_qualify {
        ast::Type::Named(type_name, span) => {
            let qualified_type_name = definitions.lookup_type(type_name)?;

            Ok(Type::Named {
                name: qualified_type_name,
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
    }
}

pub fn field<'old, 'new, 'ident>(
    to_qualify: ast::Field<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Field<'new, 'ident>> {
    let qualified_value = expr(to_qualify.value, definitions, interner, general)?;

    Ok(Field {
        name: to_qualify.name,
        value: qualified_value,
        span: to_qualify.span,
    })
}

pub fn branch<'old, 'new, 'ident>(
    to_qualify: ast::Branch<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Branch<'new, 'ident>> {
    let qualified_pattern = pattern(to_qualify.pattern, definitions, interner, general)?;
    let qualified_body = expr(to_qualify.body, definitions, interner, general)?;

    Ok(Branch {
        pattern: qualified_pattern,
        body: qualified_body,
        span: to_qualify.span,
    })
}

pub fn expr<'old, 'new, 'ident>(
    to_qualify: ast::Expr<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'new, 'ident, Expr<'new, 'ident>> {
    match to_qualify {
        ast::Expr::Variable(variable, span) => {
            let qualified_variable = definitions.lookup_variable(variable)?;
            Ok(Expr::Variable(qualified_variable, span))
        }
        ast::Expr::Literal(lit, span) => {
            let qualified_lit = lit.realloc(general);
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
        ast::Expr::StructLiteral { name, fields, span } => {
            let definition = definitions.lookup_struct(name)?;
            let qualified_fields = general.alloc_slice_try_fill_iter(
                fields
                    .iter()
                    .map(|f| field(*f, definitions, interner, general)),
            )?;

            let name_identifier = Identifier {
                source: definition.source,
                name,
                r#type: Some(Type::Named {
                    name: TypeName {
                        source: definition.source,
                        name,
                    },
                    span,
                }),
            };

            struct_contains_fields(qualified_fields, definition.fields)?;

            Ok(Expr::StructLiteral {
                name: name_identifier,
                fields: qualified_fields,
                span,
            })
        }
        ast::Expr::Block(statements) => {
            let qualified_block = block(statements, definitions, interner, general)?;
            Ok(Expr::Block(qualified_block))
        }
        ast::Expr::Annotated { .. } => todo!(),
        ast::Expr::Case {
            predicate,
            branches,
            span,
        } => {
            let qualified_predicate =
                general.alloc(expr(*predicate, definitions, interner, general)?);

            let qualified_branches = general.alloc_slice_try_fill_iter(
                branches
                    .iter()
                    .map(|b| branch(*b, definitions, interner, general)),
            )?;

            Ok(Expr::Case {
                predicate: qualified_predicate,
                branches: qualified_branches,
                span,
            })
        }
    }
}

pub fn struct_contains_fields<'expr, 'ident>(
    to_check: &'expr [Field<'expr, 'ident>],
    must_have: &[FieldDefinition<'expr, 'ident>],
) -> Result<'expr, 'ident, ()> {
    for required_field in must_have {
        let missing_field = to_check
            .iter()
            .find(|field| field.name == required_field.name)
            .is_none();
        if missing_field {
            return Err(Error::StructLiteralMissingField(*required_field, to_check));
        }
    }

    Ok(())
}
