use crate::env::Env;
use arena_alloc::General;
use ir::ast::Literal;
use ir::qualified::{self, Type};
use ir::typed::*;
use std::result;

#[derive(Debug)]
pub enum Error {}

type Result<T> = result::Result<T, Error>;

pub fn program<'old, 'new, 'ident>(
    to_infer: qualified::Program<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Program<'new, 'ident>> {
    todo!()
}

pub fn definition<'old, 'new, 'ident>(
    to_infer: qualified::Definition<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Definition<'new, 'ident>> {
    todo!()
}

pub fn argument<'old, 'new, 'ident>(
    to_infer: qualified::Argument<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Argument<'new, 'ident>> {
    todo!()
}

pub fn block<'old, 'new, 'ident>(
    to_infer: qualified::Block<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Block<'new, 'ident>> {
    todo!()
}

pub fn statement<'old, 'new, 'ident>(
    to_infer: qualified::Statement<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Statement<'new, 'ident>> {
    todo!()
}

pub fn pattern_field<'old, 'new, 'ident>(
    to_infer: qualified::PatternField<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<PatternField<'new, 'ident>> {
    todo!()
}

pub fn pattern<'old, 'new, 'ident>(
    to_infer: qualified::Pattern<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Pattern<'new, 'ident>> {
    todo!()
}

pub fn type_field<'old, 'new, 'ident>(
    to_infer: qualified::TypeField<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<qualified::TypeField<'new, 'ident>> {
    todo!()
}

pub fn r#type<'old, 'new, 'ident>(
    to_infer: Type<'old, 'ident>,
    alloc: General<'new>,
) -> Result<Type<'new, 'ident>> {
    todo!()
}

pub fn literal<'old, 'new, 'ident>(to_infer: Literal<'old>) -> Result<Literal<'new>> {
    todo!()
}

pub fn field<'old, 'new, 'ident>(
    to_infer: qualified::Field<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Field<'new, 'ident>> {
    todo!()
}

pub fn branch<'old, 'new, 'ident>(
    to_infer: qualified::Branch<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Branch<'new, 'ident>> {
    todo!()
}

pub fn expr<'old, 'new, 'ident>(
    to_infer: qualified::Expr<'old, 'ident>,
    env: &mut Env<'new, 'ident>,
    alloc: General<'new>,
) -> Result<Expr<'new, 'ident>> {
    todo!()
}
