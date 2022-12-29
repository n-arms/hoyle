use crate::util::*;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Type, TypeField};
use ir::token::{Kind, Token};
use std::iter::Peekable;

pub fn named_type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let (name, span) = propogate!(identifier(text, interner));
    Ok(Ok(Type::Named(name, span)))
}

pub fn arrow_type<'src, 'ident, 'expr>(
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

pub fn variant_type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let (tag, start) = propogate!(variant_tag(text, interner));
    let mut end = start;
    let mut args = Vec::new();
    loop {
        match not_union(text, alloc, interner)? {
            Ok(arg) => {
                end = arg.span();
                args.push(arg);
            }
            Err(_) => {
                return Ok(Ok(Type::Variant {
                    tag,
                    arguments: alloc.alloc_slice_fill_iter(args),
                    span: start.union(&end),
                }));
            }
        }
    }
}

pub fn type_field<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<TypeField<'expr, 'ident>> {
    let (name, start) = propogate!(identifier(text, interner));

    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingField)?;

    let field_type = r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingField)?;

    Ok(Ok(TypeField {
        name,
        field_type,
        span: start.union(&field_type.span()),
    }))
}

pub fn record_type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let (fields, span) = propogate!(list(
        text,
        alloc,
        interner,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut type_field,
        true
    ));

    Ok(Ok(Type::Record { fields, span }))
}

pub fn not_union<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    or_try(
        or_try(
            or_try(
                arrow_type(text, alloc, interner),
                record_type(text, alloc, interner),
            ),
            variant_type(text, alloc, interner),
        ),
        named_type(text, interner),
    )
}

pub fn r#type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let first = propogate!(not_union(text, alloc, interner));
    let start = first.span();
    let mut end = start;
    let mut cases = vec![first];

    while token(text, Kind::SingleBar)?.is_ok() {
        let case =
            not_union(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingUnionType)?;
        end = case.span();
        cases.push(case);
    }

    if cases.len() == 1 {
        Ok(Ok(cases[0]))
    } else {
        Ok(Ok(Type::Union {
            cases: alloc.alloc_slice_fill_iter(cases),
            span: start.union(&end),
        }))
    }
}
