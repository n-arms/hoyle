use crate::util::{identifier, list, or_try, propagate, token, Irrecoverable, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::source::Type;
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn named<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
) -> Result<Type<'expr>> {
    let (name, span) = propagate!(identifier(text,));
    Ok(Ok(Type::Named { name, span }))
}

fn arrow<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Type<'expr>> {
    let start = propagate!(token(text, Kind::Func));

    let (arguments, _) = list(
        text,
        alloc,
        Kind::LeftParen,
        Kind::RightParen,
        &mut r#type,
        false,
    )?
    .map_err(Irrecoverable::WhileParsingArrowType)?;

    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingArrowType)?;
    let return_type = r#type(text, alloc)?.map_err(Irrecoverable::WhileParsingArrowType)?;

    Ok(Ok(Type::Function {
        arguments,
        span: return_type.span().union(&start.into()),
        return_type: alloc.alloc(return_type),
    }))
}

pub fn r#type<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Type<'expr>> {
    or_try!(arrow(text, alloc,), named(text,))
}
