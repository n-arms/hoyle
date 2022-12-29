use crate::util::*;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Pattern, PatternField};
use ir::token::{Kind, Token};
use std::iter::Peekable;

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

pub fn pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    let (name, start) = propogate!(identifier(text, interner));

    let field_list = list(
        text,
        alloc,
        interner,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut pattern_field,
        false,
    )?;

    if let Ok((fields, end)) = field_list {
        Ok(Ok(Pattern::Struct {
            fields,
            span: end.union(&start.into()),
        }))
    } else {
        Ok(Ok(Pattern::Variable(name, start)))
    }
}
