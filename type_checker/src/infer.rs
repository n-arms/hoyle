use crate::{check, env::Env, error::Result, extract::Typeable};
use arena_alloc::General;
use ir::{qualified, typed};

pub fn program<'old, 'new>(
    to_infer: qualified::Program<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Program<'new>> {
    let typed_definitions = general.alloc_slice_try_fill_iter(
        to_infer
            .definitions
            .iter()
            .map(|def| definition(def, env, general)),
    )?;

    Ok(typed::Program {
        definitions: typed_definitions,
    })
}

pub fn definition<'old, 'new>(
    to_infer: &qualified::Definition<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Definition<'new>> {
    match to_infer {
        qualified::Definition::Function(function) => {
            let typed_definition = function_definition(function, env, general)?;
            Ok(typed::Definition::Function(typed_definition))
        }
        qualified::Definition::Struct(r#struct) => {
            let typed_definition = struct_definition(r#struct, env, general)?;
            Ok(typed::Definition::Struct(typed_definition))
        }
    }
}

pub fn function_definition<'old, 'new>(
    to_infer: &qualified::FunctionDefinition<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::FunctionDefinition<'new>> {
    let typed_generics = general.alloc_slice_fill_iter(to_infer.generics.iter().map(|generic| {
        typed::GenericDefinition {
            name: generic.name.clone(),
        }
    }));
    let typed_arguments = general.alloc_slice_try_fill_iter(
        to_infer
            .arguments
            .iter()
            .map(|arg| argument_definition(arg, env, general)),
    )?;
    let return_type = r#type(&to_infer.return_type, env, general)?;
    let body = expr(&to_infer.body, env, general)?;
    Ok(typed::FunctionDefinition {
        name: to_infer.name.clone(),
        generics: typed_generics,
        arguments: typed_arguments,
        return_type,
        body,
    })
}

pub fn struct_definition<'old, 'new>(
    to_infer: &qualified::StructDefinition<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::StructDefinition<'new>> {
    todo!()
}

pub fn argument_definition<'old, 'new>(
    to_infer: &qualified::ArgumentDefinition<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::ArgumentDefinition<'new>> {
    let typed_type = r#type(&to_infer.r#type, env, general)?;
    let typed_pattern = check::pattern(&to_infer.pattern, &typed_type, env, general)?;
    Ok(typed::ArgumentDefinition {
        pattern: typed_pattern,
        r#type: typed_type,
    })
}

pub fn r#type<'old, 'new>(
    to_infer: &qualified::Type<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Type<'new>> {
    match to_infer {
        qualified::Type::Named { name, span } => Ok(typed::Type::Named {
            name: name.clone(),
            arguments: &[],
        }),
        qualified::Type::Generic { name, span } => Ok(typed::Type::Generic { name: name.clone() }),
        qualified::Type::Function {
            arguments,
            return_type,
            span,
        } => {
            let typed_arguments = general
                .alloc_slice_try_fill_iter(arguments.iter().map(|arg| r#type(arg, env, general)))?;
            let typed_return_type = r#type(return_type, env, general)?;
            Ok(typed::Type::Function {
                arguments: typed_arguments,
                return_type: general.alloc(typed_return_type),
            })
        }
        qualified::Type::Application {
            main,
            arguments,
            span,
        } => {
            todo!()
        }
    }
}

pub fn expr<'old, 'new>(
    to_infer: &qualified::Expr<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Expr<'new>> {
    match to_infer {
        qualified::Expr::Literal { literal, span } => {
            let typed_literal = match literal {
                qualified::Literal::Boolean(bool) => typed::Literal::Boolean(*bool),
                qualified::Literal::Integer(int) => typed::Literal::Integer(*int),
            };
            Ok(typed::Expr::Literal {
                literal: typed_literal,
            })
        }
        qualified::Expr::Variable { identifier, span } => {
            let scheme = env.lookup_variable(identifier);

            assert!(scheme.for_all.is_empty()); // todo: handle generics :)

            Ok(typed::Expr::Variable {
                identifier: identifier.clone(),
                r#type: scheme.r#type.clone(),
                specialized_to: &[],
            })
        }
        qualified::Expr::Call {
            function,
            arguments,
            span,
        } => {
            let typed_function = general.alloc(expr(function, env, general)?);
            let typed_arguments = general
                .alloc_slice_try_fill_iter(arguments.iter().map(|arg| expr(arg, env, general)))?;
            let argument_types = general.alloc_slice_fill_iter(
                typed_arguments
                    .iter()
                    .map(|arg| arg.extract(&env.primitives)),
            );
            let return_type = match typed_function.extract(&env.primitives) {
                typed::Type::Function { return_type, .. } => return_type.clone(),
                r#type => return Err(todo!()),
            };
            let typed_type = typed::Type::Function {
                arguments: argument_types,
                return_type: general.alloc(return_type),
            };
            Ok(typed::Expr::Call {
                function: typed_function,
                arguments: typed_arguments,
                r#type: typed_type,
            })
        }
        qualified::Expr::Operation {
            operator,
            arguments,
            span,
        } => {
            let typed_operation = match operator {
                qualified::Operation::Add => typed::Operation::Add,
            };
            let typed_arguments = general
                .alloc_slice_try_fill_iter(arguments.iter().map(|arg| expr(arg, env, general)))?;
            let return_type = match operator {
                qualified::Operation::Add => typed::Type::Named {
                    name: env.primitives.integer.clone(),
                    arguments: &[],
                },
            };
            Ok(typed::Expr::Operation {
                operation: typed_operation,
                arguments: typed_arguments,
                r#type: return_type,
            })
        }
        qualified::Expr::StructLiteral { name, fields, span } => {
            let typed_fields =
                general.alloc_slice_try_fill_iter(fields.iter().map(|f| field(f, env, general)))?;

            let struct_name = env.lookup_struct(name).name.clone();
            let r#type = typed::Type::Named {
                name: struct_name,
                arguments: todo!(),
            };

            Ok(typed::Expr::StructLiteral {
                name: name.clone(),
                fields: typed_fields,
                r#type,
            })
        }
        qualified::Expr::Block(b) => Ok(typed::Expr::Block(block(b, env, general)?)),
        qualified::Expr::Annotated {
            expr: e,
            annotation,
            span,
        } => Ok(typed::Expr::Annotated {
            expr: general.alloc(expr(e, env, general)?),
            annotation: r#type(annotation, env, general)?,
        }),
        qualified::Expr::Case {
            predicate,
            branches,
            span,
        } => todo!(),
    }
}

pub fn field<'old, 'new>(
    to_infer: &qualified::Field<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Field<'new>> {
    let typed_value = expr(&to_infer.value, env, general)?;
    Ok(typed::Field {
        name: to_infer.name.clone(),
        value: typed_value,
    })
}

pub fn block<'old, 'new>(
    to_infer: &qualified::Block<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Block<'new>> {
    let result: Option<&_> = if let Some(result) = to_infer.result {
        Some(general.alloc(expr(result, env, general)?))
    } else {
        None
    };
    Ok(typed::Block {
        statements: general.alloc_slice_try_fill_iter(
            to_infer
                .statements
                .iter()
                .map(|stmt| statement(stmt, env, general)),
        )?,
        result,
    })
}

pub fn statement<'old, 'new>(
    to_infer: &qualified::Statement<'old>,
    env: &mut Env<'new>,
    general: &General<'new>,
) -> Result<'new, typed::Statement<'new>> {
    match to_infer {
        qualified::Statement::Raw(inner) => Ok(typed::Statement::Raw(expr(inner, env, general)?)),
        qualified::Statement::Let {
            pattern: p,
            value,
            span,
        } => {
            let typed_value = expr(value, env, general)?;
            Ok(typed::Statement::Let {
                pattern: check::pattern(p, &typed_value.extract(&env.primitives), env, general)?,
                value: typed_value,
            })
        }
    }
}
/*
use crate::check::{branch, field, pattern};
use crate::env::Env;
use crate::error::Result;
use crate::extract::{struct_type, Typeable};
use crate::substitute::{Substitute, Substitution};
use crate::unify::substitute_types;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::Literal;
use ir::qualified::{self, Type};
use ir::typed::{
    Argument, Block, Definition, Expr, FieldDefinition, Identifier, Program, Statement,
};

pub fn program<'old, 'new, 'names>(
    to_infer: qualified::Program<'old>,
    env: &mut Env<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Program<'new>> {
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

pub fn field_definition<'old, 'new>(
    to_infer: qualified::FieldDefinition<'old>,
    general: &General<'new>,
) -> Result<'new, FieldDefinition<'new>> {
    let typed_field_type = r#type(to_infer.field_type, general);
    let name = Identifier {
        identifier: to_infer.name,
        r#type: typed_field_type,
    };

    Ok(FieldDefinition {
        name,
        field_type: typed_field_type,
        span: to_infer.span,
    })
}

fn arrow_type<'expr, I>(
    arguments: I,
    r#return: Type<'expr>,
    general: &General<'expr>,
) -> Type<'expr>
where
    I: IntoIterator<Item = Type<'expr>>,
    I::IntoIter: ExactSizeIterator,
{
    Type::Arrow {
        arguments: general.alloc_slice_fill_iter(arguments),
        return_type: general.alloc(r#return),
        span: None,
    }
}

pub fn definition<'old, 'new, 'names>(
    to_infer: qualified::Definition<'old>,
    env: &mut Env<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Definition<'new>> {
    match to_infer {
        ir::ast::Definition::Function {
            name,
            generics,
            arguments,
            return_type,
            body,
            span,
        } => {
            let typed_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| argument(*arg, env, interner, general)),
            )?;

            let return_type = return_type.map(|return_type| r#type(return_type, general));

            let typed_body;
            let mut substitution = Substitution::default();

            let forall = generics.iter().map(|generic| generic.identifier);

            if let Some(return_type) = return_type {
                let func_type = Type::Arrow {
                    arguments: general.alloc_slice_fill_iter(
                        typed_arguments.iter().map(|arg| arg.type_annotation),
                    ),
                    return_type: general.alloc(return_type),
                    span: None,
                };
                env.bind_qualified_variable(name, func_type, forall.collect());
                typed_body = expr(body, env, &mut substitution, interner, general)?
                    .apply(&substitution, general);
            } else {
                typed_body = expr(body, env, &mut substitution, interner, general)?
                    .apply(&substitution, general);
                env.bind_qualified_variable(
                    name,
                    typed_body.extract(&env.primitives),
                    forall.collect(),
                );
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

pub fn argument<'old, 'new, 'names>(
    to_infer: qualified::Argument<'old>,
    env: &mut Env<'new, 'names>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Argument<'new>> {
    let type_annotation = r#type(to_infer.type_annotation, general);
    let typed_pattern = pattern(to_infer.pattern, type_annotation, env, interner, general)?;

    Ok(Argument {
        pattern: typed_pattern,
        type_annotation,
        span: to_infer.span,
    })
}

pub fn block<'old, 'new, 'names>(
    to_infer: qualified::Block<'old>,
    env: &mut Env<'new, 'names>,
    substitution: &mut Substitution<'new>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Block<'new>> {
    let typed_statements = general.alloc_slice_try_fill_iter(
        to_infer
            .statements
            .iter()
            .map(|stmt| statement(*stmt, env, substitution, interner, general)),
    )?;

    let typed_result = if let Some(result) = to_infer.result {
        let alloc_result: &_ = general.alloc(expr(*result, env, substitution, interner, general)?);
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

pub fn statement<'old, 'new, 'names>(
    to_infer: qualified::Statement<'old>,
    env: &mut Env<'new, 'names>,
    substitution: &mut Substitution<'new>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Statement<'new>> {
    match to_infer {
        ir::ast::Statement::Let {
            left_side,
            right_side,
            span,
        } => {
            let typed_right_side = expr(right_side, env, substitution, interner, general)?;

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
            let typed_raw = expr(raw, env, substitution, interner, general)?;

            Ok(Statement::Raw(typed_raw, span))
        }
    }
}

#[must_use]
pub fn r#type<'old, 'new>(to_infer: qualified::Type<'old>, general: &General<'new>) -> Type<'new> {
    match to_infer {
        qualified::Type::Named { name, span } => Type::Named { name, span },
        qualified::Type::Arrow {
            arguments,
            return_type,
            span,
        } => Type::Arrow {
            arguments: general
                .alloc_slice_fill_iter(arguments.iter().map(|arg| r#type(*arg, general))),
            return_type: general.alloc(r#type(*return_type, general)),
            span,
        },
    }
}

pub fn literal<'old, 'new>(
    to_infer: Literal<'old>,
    _interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Literal<'new>> {
    match to_infer {
        Literal::Integer(integer) => Ok(Literal::Integer(general.alloc_str(integer))),
    }
}

pub fn expr<'old, 'new, 'names>(
    to_infer: qualified::Expr<'old>,
    env: &mut Env<'new, 'names>,
    substitution: &mut Substitution<'new>,
    interner: &Interning<Specialized>,
    general: &General<'new>,
) -> Result<'new, Expr<'new>> {
    match to_infer {
        ir::ast::Expr::Variable(variable, span) => {
            let typed_variable = env.lookup_variable(variable, general);

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
            let typed_function = expr(*function, env, substitution, interner, general)?;
            let typed_arguments = general.alloc_slice_try_fill_iter(
                arguments
                    .iter()
                    .map(|arg| expr(*arg, env, substitution, interner, general)),
            )?;
            if let Type::Arrow { arguments, .. } = typed_function.extract(&env.primitives) {
                for (expected, found) in arguments.iter().zip(typed_arguments.iter()) {
                    substitution.union(&substitute_types(
                        *expected,
                        found.extract(&env.primitives),
                    )?);
                }
            } else {
                todo!()
            }
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
            let typed_block = block(untyped_block, env, substitution, interner, general)?;

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
            let typed_predicate = expr(*predicate, env, substitution, interner, general)?;

            let typed_branches = general.alloc_slice_try_fill_iter(branches.iter().map(|b| {
                branch(
                    *b,
                    typed_predicate.extract(&env.primitives),
                    env,
                    substitution,
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

            let typed_fields =
                general.alloc_slice_try_fill_iter(fields.iter().map(|to_check| {
                    field(
                        *to_check,
                        defined_type,
                        env,
                        substitution,
                        interner,
                        general,
                    )
                }))?;

            Ok(Expr::StructLiteral {
                name: typed_name,
                fields: typed_fields,
                span,
            })
        }
    }
}
*/
