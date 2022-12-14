use crate::expr::expr;
use crate::pattern::pattern;
use crate::types::r#type;
use crate::util::{identifier, list, or_try, propagate, token, Irrecoverable, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Argument, Definition, FieldDefinition, Generic, Program, Type};
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn generic<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    _alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Generic<&'ident str>> {
    let (identifier, span) = propagate!(identifier(text, interner));
    Ok(Ok(Generic { identifier, span }))
}

fn argument<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Argument<'expr, &'ident str, &'ident str>> {
    let pattern = propagate!(pattern(text, alloc, interner));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingArgument)?;
    let type_annotation =
        r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingArgument)?;

    Ok(Ok(Argument {
        pattern,
        type_annotation,
        span: pattern.span().union(&type_annotation.span().unwrap()),
    }))
}

fn generics<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<&'expr [Generic<&'ident str>]> {
    let (generics, _) = propagate!(list(
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
) -> Result<&'expr [Argument<'expr, &'ident str, &'ident str>]> {
    let (arguments, _) = propagate!(list(
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
) -> Result<Type<'expr, &'ident str>> {
    let _ = propagate!(token(text, Kind::Colon));
    let r#type = r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingReturnType)?;
    Ok(Ok(r#type))
}

fn field_definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<FieldDefinition<'expr, &'ident str, &'ident str>> {
    let (name, start) = propagate!(identifier(text, interner));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingFieldDefinition)?;
    let field_type =
        r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingFieldDefinition)?;
    let span = field_type.span().unwrap().union(&start);

    Ok(Ok(FieldDefinition {
        name,
        field_type,
        span,
    }))
}

fn struct_definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Definition<'expr, &'ident str, &'ident str>> {
    let start = propagate!(token(text, Kind::Struct));
    let (name, _) = identifier(text, interner)?.map_err(Irrecoverable::WhileParsingStruct)?;
    let (fields, end) = list(
        text,
        alloc,
        interner,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut field_definition,
        false,
    )?
    .map_err(Irrecoverable::WhileParsingStruct)?;

    let span = end.union(&start.into());

    Ok(Ok(Definition::Struct { name, fields, span }))
}

fn function_definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Definition<'expr, &'ident str, &'ident str>> {
    let start = propagate!(token(text, Kind::Func));
    let (name, _) = identifier(text, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let generics = generics(text, alloc, interner)?.unwrap_or_default();
    let arguments = arguments(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let return_type = return_type(text, alloc, interner)?.ok();
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let body = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;

    Ok(Ok(Definition::Function {
        name,
        generics,
        arguments,
        return_type,
        body,
        span: body.span().union(&start.into()),
    }))
}

fn definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Definition<'expr, &'ident str, &'ident str>> {
    or_try!(
        struct_definition(text, alloc, interner),
        function_definition(text, alloc, interner)
    )
}

pub fn program<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Program<'expr, &'ident str, &'ident str>> {
    let mut defs = Vec::new();

    while text.peek().is_some() {
        let def = definition(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingProgram)?;
        defs.push(def);
    }

    Ok(Ok(Program {
        definitions: alloc.alloc_slice_fill_iter(defs),
    }))
}
