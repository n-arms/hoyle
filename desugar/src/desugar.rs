use crate::builder;
use crate::metadata::Env;
use arena_alloc::General;
use ir::desugared::*;
use ir::qualified::{self, Identifier};
use ir::typed;

pub fn expr<'old, 'new, 'names, 'ident>(
    to_desugar: typed::Expr<'old, 'ident>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'ident>,
    alloc: &General<'new>,
) -> Atom {
    match to_desugar {
        ir::ast::Expr::Variable(variable, _) => {
            let name = env.lookup_identifier(variable);
            Atom::Variable(name)
        }
        ir::ast::Expr::Literal(ir::ast::Literal::Integer(int), _) => {
            Atom::Literal(Literal::Integer(
                int.parse()
                    .expect("the parser should have caught improper integer literals"),
            ))
        }
        ir::ast::Expr::Call {
            function,
            arguments,
            ..
        } => todo!(),
        ir::ast::Expr::Operation {
            operator,
            arguments,
            span,
        } => todo!(),
        ir::ast::Expr::StructLiteral { name, fields, span } => {
            let type_name = env.lookup_identifier(name);
            let desugared_fields =
                alloc.alloc_slice_fill_iter(fields.iter().map(|f| field(*f, block, env, alloc)));
            let result = block.fresh_name();
            block.with_statement(Statement {
                variable: result,
                r#type: Type::Named { name: type_name },
                value: Expr::Struct {
                    fields: desugared_fields,
                },
            });
            Atom::Variable(result)
        }
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

pub fn field<'old, 'new, 'names, 'ident>(
    to_desugar: typed::Field<'old, 'ident>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'ident>,
    alloc: &General<'new>,
) -> Field {
    let desugared_name = env.lookup_identifier(to_desugar.name);
    let desugared_value = expr(to_desugar.value, block, env, alloc);

    Field {
        name: desugared_name,
        value: desugared_value,
    }
}
