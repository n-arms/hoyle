/*
use crate::env::*;
use im::HashSet;
use infinite_iterator::InfiniteIterator;
use ir::ast::{self, Argument, Definition, Expr, Generic, Literal, Pattern, Program, Statement};
use std::hash::Hash;
use std::result;

#[derive(Debug)]
pub enum Error {}

type Result<T> = result::Result<T, Error>;

pub fn program<'old, 'new, 'ident, S>(
    to_infer: &Program<'old, 'ident>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<Program<'new, TypedId<'new, ID>, Type<'new, ID>>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    Ok(Program {
        definitions: alloc.try_slice_fill_iter(
            to_infer
                .definitions
                .iter()
                .map(|def| definition(def, env.clone(), fresh, alloc)),
        )?,
    })
}

pub fn definition<'old, 'new, 'ident, S>(
    to_infer: &Definition<'old, 'ident>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<Definition<'new, TypedId<'new, ID>, Type<'new, ID>>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    let generics = alloc.try_slice_fill_iter(
        to_infer
            .generics
            .iter()
            .map(|gen| generic(gen, env.clone(), fresh, alloc)),
    )?;
    let arguments = alloc.try_slice_fill_iter(
        to_infer
            .arguments
            .iter()
            .map(|arg| argument(arg, arg.type_annotation.clone(), env.clone(), fresh, alloc)),
    )?;

    let return_type = if let Some(return_type) = &to_infer.return_type {
        Some(r#type(return_type, env.clone(), fresh, alloc)?)
    } else {
        None
    };

    let inner_env = env
        .declare_generics(
            to_infer
                .generics
                .iter()
                .map(|generic| generic.identifier.clone()),
        )
        .bind_variables(
            arguments
                .iter()
                .map(|arg| &arg.pattern)
                .flat_map(pattern_identifiers),
        );
    let body = expr(&to_infer.body, inner_env, fresh, alloc)?;

    Ok(Definition {
        name: todo!("add function type"),
        generics,
        arguments,
        return_type,
        body,
        span: to_infer.span,
    })
}

pub fn statement<'old, 'new, 'ident, S>(
    to_infer: &Statement<'old, ID>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<(Statement<'new, TypedId<'new, ID>>, Env<'new, &'ident str>)>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    match to_infer {
        Statement::Let {
            left_side,
            right_side,
            span,
        } => {
            let right_side = expr(right_side, env.clone(), fresh, alloc)?;
            let left_side = pattern(
                left_side,
                expr_type(&right_side, alloc),
                env.clone(),
                fresh,
                alloc,
            )?;
            let bound_env = env.bind_variables(pattern_identifiers(&left_side));
            todo!()
        }
        Statement::Raw(raw, span) => Ok((
            Statement::Raw(expr(raw, env.clone(), fresh, alloc)?, *span),
            env,
        )),
    }
}

fn expr_type<'expr, ID: Clone>(
    expr: &Expr<'expr, TypedId<'expr, ID>>,
    alloc: Alloc<'expr>,
) -> Type<'expr, ID> {
    match expr {
        Expr::Variable(TypedId { id_type, .. }, _) => id_type.clone(),
        Expr::Literal(literal, _) => literal_type(*literal, alloc),
        Expr::Call {
            function,
            arguments,
            span,
        } => todo!(),
        Expr::Operation {
            operator,
            arguments,
            span,
        } => todo!(),
        Expr::Block(_) => todo!(),
    }
}

fn int_type<'expr, ID>(alloc: Alloc<'expr>) -> Type<'expr, ID> {
    todo!()
}

fn literal_type<'expr, ID>(literal: Literal, alloc: Alloc<'expr>) -> Type<'expr, ID> {
    match literal {
        Literal::Integer(_) => int_type(alloc),
    }
}

pub fn expr<'old, 'new, 'ident, S>(
    to_infer: &Expr<'old, ID>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<Expr<'new, TypedId<'new, ID>>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn pattern<'old, 'new, 'ident, S>(
    to_infer: &Pattern<'old, ID>,
    pattern_type: Type<'old, ID>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<Pattern<'new, TypedId<'new, ID>>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

fn pattern_identifiers<'expr, ID>(pattern: &Pattern<'expr, ID>) -> HashSet<ID>
where
    ID: Hash + Eq + Clone,
{
    match pattern {
        Pattern::Variable(id, _) => HashSet::unit(id.clone()),
        Pattern::Tuple(_, _) => todo!(),
    }
}

pub fn argument<'old, 'new, 'ident, S>(
    to_infer: &Argument<'old, 'ident>,
    pattern_type: ast::Type<'old, ID>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<Argument<'new, TypedId<'new, ID>, Type<'new, ID>>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn r#type<'old, 'new, 'ident, S>(
    to_infer: &ast::Type<'old, ID>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<Type<'new, ID>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
    todo!()
}

pub fn generic<'new, ID, S>(
    to_infer: &Generic<ID>,
    env: Env<'new, &'ident str>,
    fresh: &mut Fresh<&'ident str, S>,
    alloc: Alloc<'new>,
) -> Result<Generic<TypedId<'new, ID>>>
where
    ID: Hash + Eq + Clone,
    S: InfiniteIterator<Item = ID>,
{
}
*/
