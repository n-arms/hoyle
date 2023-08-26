use crate::expr::expr;
use crate::pattern::pattern;
use crate::types::r#type;
use crate::util::{identifier, list, or_try, propagate, token, Irrecoverable, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::source::{
    ArgumentDefinition, Definition, FieldDefinition, FunctionDefinition, GenericDefinition,
    Program, StructDefinition, Type,
};
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn generic<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    _alloc: &General<'expr>,
) -> Result<GenericDefinition> {
    let (name, _) = propagate!(identifier(text));
    Ok(Ok(GenericDefinition { name }))
}

fn argument<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<ArgumentDefinition<'expr>> {
    let pattern = propagate!(pattern(text, alloc));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingArgument)?;
    let r#type = r#type(text, alloc)?.map_err(Irrecoverable::WhileParsingArgument)?;

    Ok(Ok(ArgumentDefinition {
        span: pattern.span().union(&r#type.span()),
        pattern,
        r#type,
    }))
}

fn generics<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<&'expr [GenericDefinition]> {
    let (generics, _) = propagate!(list(
        text,
        alloc,
        Kind::LeftSquareBracket,
        Kind::RightSquareBracket,
        &mut generic,
        false
    ));
    Ok(Ok(generics))
}

fn arguments<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<&'expr [ArgumentDefinition<'expr>]> {
    let (arguments, _) = propagate!(list(
        text,
        alloc,
        Kind::LeftParen,
        Kind::RightParen,
        &mut argument,
        false
    ));
    Ok(Ok(arguments))
}

fn return_type<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Type<'expr>> {
    let _ = propagate!(token(text, Kind::Colon));
    let r#type = r#type(text, alloc)?.map_err(Irrecoverable::WhileParsingReturnType)?;
    Ok(Ok(r#type))
}

fn field_definition<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<FieldDefinition<'expr>> {
    let (field, start) = propagate!(identifier(text));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingFieldDefinition)?;
    let r#type = r#type(text, alloc)?.map_err(Irrecoverable::WhileParsingFieldDefinition)?;
    let span = r#type.span().union(&start);

    Ok(Ok(FieldDefinition {
        field,
        r#type,
        span,
    }))
}

fn struct_definition<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<StructDefinition<'expr>> {
    let start = propagate!(token(text, Kind::Struct));
    let (name, _) = identifier(text)?.map_err(Irrecoverable::WhileParsingStruct)?;
    let (fields, end) = list(
        text,
        alloc,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut field_definition,
        false,
    )?
    .map_err(Irrecoverable::WhileParsingStruct)?;

    let span = end.union(&start.into());

    Ok(Ok(StructDefinition { name, fields, span }))
}

fn function_definition<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<FunctionDefinition<'expr>> {
    let start = propagate!(token(text, Kind::Func));
    let (name, _) = identifier(text)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let generics = generics(text, alloc)?.unwrap_or_default();
    let arguments = arguments(text, alloc)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let return_type = return_type(text, alloc)?.map_err(Irrecoverable::WhileParsingReturnType)?;
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let body = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingFunc)?;

    Ok(Ok(FunctionDefinition {
        name,
        generics,
        arguments,
        return_type,
        span: body.span().union(&start.into()),
        body,
    }))
}

fn definition<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Definition<'expr>> {
    or_try!(
        struct_definition(text, alloc)
            .map(|res| res.map(|struct_def| Definition::Struct(struct_def))),
        function_definition(text, alloc)
            .map(|res| res.map(|func_def| Definition::Function(func_def)))
    )
}

pub fn program<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Program<'expr>> {
    let mut defs = Vec::new();

    while text.peek().is_some() {
        let def = definition(text, alloc)?.map_err(Irrecoverable::WhileParsingProgram)?;
        defs.push(def);
    }

    Ok(Ok(Program {
        definitions: alloc.alloc_slice_fill_iter(defs),
    }))
}
