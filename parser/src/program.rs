use crate::expr::*;
use crate::pattern::*;
use crate::types::*;
use crate::util::*;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Argument, Definition, Generic, Program, Type};
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn generic<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    _alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Generic<'ident>> {
    let (identifier, span) = propogate!(identifier(text, interner));
    Ok(Ok(Generic { identifier, span }))
}

fn argument<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Argument<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let pattern = propogate!(pattern(text, alloc, interner));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingArgument)?;
    let type_annotation =
        r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingArgument)?;

    Ok(Ok(Argument {
        pattern,
        type_annotation,
        span: pattern.span().union(&type_annotation.span()),
    }))
}

fn generics<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<&'expr [Generic<'ident>]> {
    let (generics, _) = propogate!(list(
        text,
        alloc,
        interner,
        Kind::LeftSquareBracket,
        Kind::RightSquareBracket,
        &mut generic,
        false
    ));
    Ok(Ok(generics))
}

fn arguments<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<&'expr [Argument<'expr, 'ident, &'ident str, Type<'expr, 'ident>>]> {
    let (arguments, _) = propogate!(list(
        text,
        alloc,
        interner,
        Kind::LeftParen,
        Kind::RightParen,
        &mut argument,
        false
    ));
    Ok(Ok(arguments))
}

fn return_type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let _ = propogate!(token(text, Kind::Colon));
    let r#type = r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingReturnType)?;
    Ok(Ok(r#type))
}

fn definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Definition<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let start = propogate!(token(text, Kind::Func));
    let (name, _) = identifier(text, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let generics = generics(text, alloc, interner)?.unwrap_or_default();
    let arguments = arguments(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let return_type = return_type(text, alloc, interner)?.ok();
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let body = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;

    Ok(Ok(Definition {
        name,
        generics,
        arguments,
        return_type,
        body,
        span: body.span().union(&start.into()),
    }))
}

pub fn program<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Program<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let mut defs = Vec::new();

    while text.peek().is_some() {
        let def = definition(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingProgram)?;
        defs.push(def);
    }

    Ok(Ok(Program {
        definitions: alloc.alloc_slice_fill_iter(defs),
    }))
}
