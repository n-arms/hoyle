use crate::builder;
use crate::env::{Env, VariableTemplate};
use arena_alloc::General;
use ir::desugared::*;
use ir::qualified;
use ir::typed;

pub fn expr<'old, 'new, 'names, 'ident>(
    to_desugar: typed::Expr<'old, 'ident>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'ident>,
    alloc: &General<'new>,
) -> Atom {
    match to_desugar {
        ir::ast::Expr::Variable(variable, _) => {
            let template = env.lookup_variable(variable.identifier);

            let name = expand_template(template, variable.r#type, block, env, alloc);
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
        } => {
            let desugared_function = expr(*function, block, env, alloc);
            let desugared_arguments = alloc
                .alloc_slice_fill_iter(arguments.iter().map(|arg| expr(*arg, block, env, alloc)));
            let call = Expr::Call {
                function: alloc.alloc(desugared_function),
                arguments: desugared_arguments,
            };
            let result_name = block.fresh_name();
            block.with_statement(Statement {
                variable: result_name,
                r#type: todo!(),
                value: call,
            });
            Atom::Variable(result_name)
        }
        ir::ast::Expr::Operation {
            operator,
            arguments,
            ..
        } => todo!(),
        ir::ast::Expr::StructLiteral { name, fields, .. } => todo!(),
        ir::ast::Expr::Block(_) => todo!(),
        ir::ast::Expr::Annotated {
            expr, annotation, ..
        } => todo!(),
        ir::ast::Expr::Case {
            predicate,
            branches,
            ..
        } => todo!(),
    }
}

pub fn expand_template<'old, 'new, 'names, 'ident>(
    template: VariableTemplate<'old, 'ident>,
    instance_type: qualified::Type<'old, 'ident>,
    block: &mut builder::Block<'names, 'new>,
    env: &mut Env<'old, 'ident>,
    alloc: &General<'new>,
) -> Name {
    match template {
        VariableTemplate::Monomorphic { name } => name,
        VariableTemplate::Polymorphic { .. } => {
            todo!()
        }
    }
}
