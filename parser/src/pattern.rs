use crate::util::{identifier, list, propagate, token, Irrecoverable, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::source::{Pattern, PatternField};
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn pattern_field<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<PatternField<'expr>> {
    let (name, start) = propagate!(identifier(text));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingPatternField)?;
    let pattern = pattern(text, alloc)?.map_err(Irrecoverable::WhileParsingPatternField)?;
    Ok(Ok(PatternField {
        name,
        span: start.union(&pattern.span()),
        pattern,
    }))
}

pub fn pattern<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Pattern<'expr>> {
    let (name, start) = propagate!(identifier(text));

    let field_list = list(
        text,
        alloc,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut pattern_field,
        false,
    )?;

    if let Ok((fields, end)) = field_list {
        Ok(Ok(Pattern::Struct {
            name,
            fields,
            span: end.union(&start),
        }))
    } else {
        Ok(Ok(Pattern::Variable { name, span: start }))
    }
}
