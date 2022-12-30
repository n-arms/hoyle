use crate::util::{identifier, list, or_try, propogate, token, Irrecoverable, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::ast::Type;
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn named<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let (name, span) = propogate!(identifier(text, interner));
    Ok(Ok(Type::Named(name, span)))
}

fn arrow<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let start = propogate!(token(text, Kind::Func));

    let (arguments, _) = list(
        text,
        alloc,
        interner,
        Kind::LeftParen,
        Kind::RightParen,
        &mut r#type,
        false,
    )?
    .map_err(Irrecoverable::WhileParsingArrowType)?;

    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingArrowType)?;
    let return_type =
        r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingArrowType)?;

    Ok(Ok(Type::Arrow {
        arguments,
        return_type: alloc.alloc(return_type),
        span: return_type.span().union(&start.into()),
    }))
}

pub fn r#type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    or_try!(arrow(text, alloc, interner), named(text, interner))
}
