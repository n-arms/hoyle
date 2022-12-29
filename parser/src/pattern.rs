use crate::util::*;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Pattern, PatternField};
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn variable_pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    let (id, span) = propogate!(identifier(text, interner));
    Ok(Ok(Pattern::Variable(id, span)))
}

fn pattern_field<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<PatternField<'expr, 'ident, &'ident str>> {
    let (name, start) = propogate!(identifier(text, interner));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingPatternField)?;
    let pattern =
        pattern(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingPatternField)?;
    Ok(Ok(PatternField {
        name,
        pattern,
        span: start.union(&pattern.span()),
    }))
}

fn record_pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    let (fields, span) = propogate!(list(
        text,
        alloc,
        interner,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut pattern_field,
        true,
    ));

    Ok(Ok(Pattern::Record { fields, span }))
}

pub fn pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    or_try!(
        record_pattern(text, alloc, interner),
        variable_pattern(text, interner)
    )
}
