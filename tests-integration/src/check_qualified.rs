pub fn program(
    typed_program: ir::ast::Program<ir::typed::Identifier, ir::qualified::Identifier>,
) -> Result<(), String> {
    for def in typed_program.definitions {
        definition(def)?;
    }
    Ok(())
}

fn definition(
    def: &ir::ast::Definition<ir::typed::Identifier, ir::qualified::Identifier>,
) -> Result<(), String> {
    match def {
        ir::ast::Definition::Function {
            name,
            generics,
            arguments,
            return_type,
            body,
            ..
        } => {
            for generic in *generics {
                identifier(generic.identifier)?;
            }

            for arg in *arguments {
                r#type(arg.type_annotation)?;
                pattern(arg.pattern)?;
            }

            if let Some(return_type) = return_type {
                r#type(*return_type)?;
            }

            typed_identifier(name)?;
            expr(body)?;
        }
        ir::ast::Definition::Struct { name, fields, .. } => {
            typed_identifier(name)?;
            for field in *fields {
                typed_identifier(&field.name)?;
                r#type(field.field_type)?;
            }
        }
    }

    Ok(())
}

fn pattern(to_check: ir::ast::Pattern<ir::typed::Identifier>) -> Result<(), String> {
    match to_check {
        ir::ast::Pattern::Variable(variable, _) => typed_identifier(&variable),
        ir::ast::Pattern::Struct { name, fields, .. } => {
            for field in fields {
                typed_identifier(&field.name)?;
                pattern(field.pattern)?;
            }
            typed_identifier(&name)
        }
    }
}

fn expr(
    body: &ir::ast::Expr<ir::typed::Identifier, ir::qualified::Identifier>,
) -> Result<(), String> {
    match body {
        ir::ast::Expr::Variable(variable, _) => typed_identifier(variable),
        ir::ast::Expr::Literal(_, _) => Ok(()),
        ir::ast::Expr::Call {
            function,
            arguments,
            ..
        } => {
            for arg in *arguments {
                expr(arg)?;
            }
            expr(function)
        }
        ir::ast::Expr::Operation { arguments, .. } => {
            for arg in *arguments {
                expr(arg)?;
            }
            Ok(())
        }
        ir::ast::Expr::StructLiteral { name, fields, .. } => {
            typed_identifier(name)?;
            for field in *fields {
                expr(&field.value)?;
                typed_identifier(&field.name)?;
            }
            Ok(())
        }
        ir::ast::Expr::Block(block) => {
            for stmt in block.statements {
                statement(stmt)?;
            }
            if let Some(last) = block.result {
                expr(last)?;
            }
            Ok(())
        }
        ir::ast::Expr::Annotated {
            expr: inner,
            annotation,
            ..
        } => {
            expr(inner)?;
            r#type(*annotation)
        }
        ir::ast::Expr::Case {
            predicate,
            branches,
            ..
        } => {
            for branch in *branches {
                pattern(branch.pattern)?;
                expr(&branch.body)?;
            }
            expr(predicate)
        }
    }
}

fn statement(
    stmt: &ir::ast::Statement<ir::typed::Identifier, ir::qualified::Identifier>,
) -> Result<(), String> {
    match stmt {
        ir::ast::Statement::Let {
            left_side,
            right_side,
            ..
        } => {
            pattern(*left_side)?;
            expr(right_side)
        }
        ir::ast::Statement::Raw(raw, _) => expr(raw),
    }
}

fn typed_identifier(name: &ir::typed::Identifier) -> Result<(), String> {
    r#type(name.r#type)
}

fn r#type(to_check: ir::ast::Type<ir::qualified::Identifier>) -> Result<(), String> {
    match to_check {
        ir::ast::Type::Named { name, .. } => identifier(name),
        ir::ast::Type::Arrow {
            arguments,
            return_type,
            ..
        } => {
            for arg in arguments {
                r#type(*arg)?;
            }
            r#type(*return_type)
        }
    }
}

fn identifier(identifier: ir::qualified::Identifier) -> Result<(), String> {
    if identifier.name == "unification" {
        Err(format!(
            "unification type variable {:?} was never unified with a concrete type",
            identifier
        ))
    } else {
        Ok(())
    }
}
