use crate::util::*;
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Block, Branch, Expr, Field, Literal, Pattern, PatternField, Span, Statement, Type};
use ir::token::{Kind, Token};
use std::iter::Peekable;

pub fn literal<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let span = propogate!(token(text, Kind::Number));
    Ok(Ok(Expr::Literal(
        Literal::Integer(alloc.alloc_str(span.data)),
        span.into(),
    )))
}

pub fn variable<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let (var, span) = propogate!(identifier(text, interner));
    Ok(Ok(Expr::Variable(var, span)))
}

pub fn parens<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let _ = propogate!(token(text, Kind::LeftParen));
    let expr = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingParens)?;
    let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingParens)?;
    Ok(Ok(expr))
}

pub fn not_application<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    // this aspect of the grammar isn't LL(1), this will make error handling for records rather bad
    if token_hint(text, Kind::LeftBrace) {
        let mut text_copy = text.clone();

        match record(&mut text_copy, alloc, interner) {
            Ok(Ok(record)) => {
                *text = text_copy;
                Ok(Ok(record))
            }
            Ok(Err(_)) => unreachable!(),
            Err(_) => block(text, alloc, interner),
        }
    } else {
        or_try(
            variable(text, interner),
            or_try(literal(text, alloc), parens(text, alloc, interner)),
        )
    }
}

pub fn not_variant<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let func = propogate!(not_application(text, alloc, interner));
    let mut args = Vec::new();

    while let Ok(next) = not_application(text, alloc, interner)? {
        args.push(next);
    }

    if let Some(last) = args.last() {
        let span = func.span().union(&last.span());
        Ok(Ok(Expr::Call {
            function: alloc.alloc(func),
            arguments: alloc.alloc_slice_fill_iter(args),
            span,
        }))
    } else {
        Ok(Ok(func))
    }
}

pub fn variant<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let (tag, start) = propogate!(variant_tag(text, interner));

    let mut args = Vec::new();

    while let Ok(next) = not_application(text, alloc, interner)? {
        args.push(next);
    }

    let end = if let Some(last) = args.last() {
        last.span()
    } else {
        start
    };
    let span = start.union(&end);
    Ok(Ok(Expr::Variant {
        tag,
        arguments: alloc.alloc_slice_fill_iter(args),
        span,
    }))
}

pub fn field<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Field<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let (name, start) = propogate!(identifier(text, interner));

    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingField)?;

    let value = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingField)?;

    Ok(Ok(Field {
        name,
        value,
        span: start.union(&value.span()),
    }))
}

pub fn record<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let (fields, span) = propogate!(list(
        text,
        alloc,
        interner,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut field,
        true,
    ));

    Ok(Ok(Expr::Record { fields, span }))
}

pub fn branch<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Branch<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let pattern = propogate!(pattern(text, alloc, interner));
    let _ = token(text, Kind::ThickArrow)?.map_err(Irrecoverable::WhileParsingBranch)?;
    let body = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingBranch)?;

    Ok(Ok(Branch {
        pattern,
        body,
        span: pattern.span().union(&body.span()),
    }))
}

pub fn case<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let start = propogate!(token(text, Kind::Case));
    let predicate =
        alloc.alloc(expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingCase)?);
    let _ = token(text, Kind::Of)?.map_err(Irrecoverable::WhileParsingCase)?;
    let (branches, end) = list(
        text,
        alloc,
        interner,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut branch,
        false,
    )?
    .map_err(Irrecoverable::WhileParsingCase)?;

    Ok(Ok(Expr::Case {
        predicate,
        branches,
        span: end.union(&start.into()),
    }))
}

pub fn expr<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    or_try(
        variant(text, alloc, interner),
        or_try(
            not_variant(text, alloc, interner),
            case(text, alloc, interner),
        ),
    )
}

pub fn variable_pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    let (id, span) = propogate!(identifier(text, interner));
    Ok(Ok(Pattern::Variable(id, span)))
}

pub fn pattern_field<'src, 'ident, 'expr>(
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

pub fn record_pattern<'src, 'ident, 'expr>(
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

pub fn not_variant_pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    or_try(
        record_pattern(text, alloc, interner),
        variable_pattern(text, interner),
    )
}

pub fn variant_pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    let (tag, start) = propogate!(variant_tag(text, interner));
    let mut end = start;
    let mut args = Vec::new();

    while let Ok(arg) = pattern(text, alloc, interner)? {
        end = arg.span();
        args.push(arg);
    }

    Ok(Ok(Pattern::Variant {
        tag,
        arguments: alloc.alloc_slice_fill_iter(args),
        span: start.union(&end),
    }))
}

pub fn pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, 'ident, &'ident str>> {
    or_try(
        variant_pattern(text, alloc, interner),
        not_variant_pattern(text, alloc, interner),
    )
}

pub fn r#let<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let start = propogate!(token(text, Kind::Let));
    let left_side = pattern(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingLet)?;
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingLet)?;
    let right_side = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingLet)?;

    Ok(Ok(Statement::Let {
        left_side,
        right_side,
        span: right_side.span().union(&start.into()),
    }))
}

pub fn raw<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let expr = propogate!(expr(text, alloc, interner));

    Ok(Ok(Statement::Raw(expr, expr.span())))
}

pub fn statement<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    or_try(r#let(text, alloc, interner), r#raw(text, alloc, interner))
}

#[allow(clippy::missing_panics_doc)]
pub fn block<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let start = propogate!(token(text, Kind::LeftBrace));
    if let Ok(end) = token(text, Kind::RightBrace)? {
        return Ok(Ok(Expr::Block(Block {
            statements: &[],
            result: None,
            span: Span::from(start).union(&end.into()),
        })));
    }
    let first = statement(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingBlock)?;
    let mut rest = vec![first];

    loop {
        if token(text, Kind::Semicolon)?.is_err() {
            let end = token(text, Kind::RightBrace)?.map_err(Irrecoverable::WhileParsingBlock)?;
            let result = match rest.pop().unwrap() {
                // will not crash because rest always has at least one element
                Statement::Raw(result, ..) => result,
                Statement::Let { .. } => return Err(Irrecoverable::MissingSemicolon(end.into())),
            };
            return Ok(Ok(Expr::Block(Block {
                statements: alloc.alloc_slice_fill_iter(rest),
                result: Some(alloc.alloc(result)),
                span: Span::from(start).union(&end.into()),
            })));
        }
        if let Ok(end) = token(text, Kind::RightBrace)? {
            return Ok(Ok(Expr::Block(Block {
                statements: alloc.alloc_slice_fill_iter(rest),
                result: None,
                span: Span::from(start).union(&end.into()),
            })));
        }
        let stmt = statement(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingBlock)?;
        rest.push(stmt);
    }
}
