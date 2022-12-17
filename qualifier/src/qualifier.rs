use crate::definitions::Definitions;
use crate::error::Result;
use arena_alloc::{General, Interning, Specialized};
use ir::ast;
use ir::qualified::*;

pub fn program<'old, 'new, 'ident>(
    to_qualify: ast::Program<'old, 'ident, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Program<'new, 'ident>> {
    let qualified_definitions = general.alloc_slice_try_fill_iter(
        to_qualify
            .definitions
            .into_iter()
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
    inner_defs.with_types(to_qualify.generics.into_iter().map(|generic| {
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
        .map(|result| r#type(result, &mut inner_defs, interner, general))
        .unwrap_or(Ok(Type::Wildcard))?;

    let arguments = general.alloc_slice_try_fill_iter(
        to_qualify
            .arguments
            .into_iter()
            .map(|arg| argument(*arg, &mut inner_defs, interner, general)),
    )?;

    let body = expr(to_qualify.body, &mut inner_defs, interner, general)?;

    let generics = general.alloc_slice_fill_iter(to_qualify.generics.into_iter().copied());

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
    to_qualify: ast::Statement<'old, &'ident str, ast::Type<'old, 'ident>>,
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
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
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
    to_qualify: ast::Block<'old, &'ident str, ast::Type<'old, 'ident>>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, Block<'new, 'ident>> {
    let statements = general.alloc_slice_try_fill_iter(
        to_qualify
            .statements
            .into_iter()
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
        ast::Type::Tuple(_, _) => todo!(),
    }
}

pub fn literal<'old, 'new, 'ident>(
    to_qualify: ast::Literal<'old>,
    definitions: &mut Definitions<'new, 'ident>,
    interner: &Interning<'ident, Specialized>,
    general: &General<'new>,
) -> Result<'ident, ast::Literal<'new>> {
    match to_qualify {
        ast::Literal::Integer(int) => Ok(ast::Literal::Integer(general.alloc_str(int))),
    }
}

pub fn expr<'old, 'new, 'ident>(
    to_qualify: ast::Expr<'old, &'ident str, ast::Type<'old, 'ident>>,
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
        } => todo!(),
        ast::Expr::Operation {
            operator,
            arguments,
            span,
        } => todo!(),
        ast::Expr::Block(_) => todo!(),
        ast::Expr::Annotated {
            expr,
            annotation,
            span,
        } => todo!(),
    }
}
