use crate::pattern::pattern;
use crate::util::{identifier, list, or_try, propagate, token, Irrecoverable, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::ast::{Block, Branch, Expr, Field, Literal, Span, Statement};
use ir::token::{Kind, Token};
use std::iter::Peekable;

fn literal<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr, &'ident str, &'ident str>> {
    let span = propagate!(token(text, Kind::Number));
    Ok(Ok(Expr::Literal(
        Literal::Integer(alloc.alloc_str(span.data)),
        span.into(),
    )))
}

fn parens<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, &'ident str, &'ident str>> {
    let _ = propagate!(token(text, Kind::LeftParen));
    let expr = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingParens)?;
    let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingParens)?;
    Ok(Ok(expr))
}

pub fn struct_or_variable<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, &'ident str, &'ident str>> {
    let identifier = propagate!(token(text, Kind::Identifier));

    let struct_list = list(
        text,
        alloc,
        interner,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut field,
        false,
    )?;

    if let Ok((fields, end)) = struct_list {
        Ok(Ok(Expr::StructLiteral {
            name: interner.get_or_intern(identifier.data),
            fields,
            span: end.union(&identifier.into()),
        }))
    } else {
        Ok(Ok(Expr::Variable(
            interner.get_or_intern(identifier.data),
            identifier.into(),
        )))
    }
}

pub fn not_application<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, &'ident str, &'ident str>> {
    or_try!(
        block(text, alloc, interner),
        literal(text, alloc),
        case(text, alloc, interner),
        parens(text, alloc, interner),
        struct_or_variable(text, alloc, interner)
    )
}

pub fn expr<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, &'ident str, &'ident str>> {
    let first = propagate!(not_application(text, alloc, interner));
    let argument_list = list(
        text,
        alloc,
        interner,
        Kind::LeftParen,
        Kind::RightParen,
        &mut expr,
        false,
    )?;
    if let Ok((arguments, end)) = argument_list {
        let span = first.span().union(&end);
        let call = Expr::Call {
            function: alloc.alloc(first),
            arguments,
            span,
        };
        Ok(Ok(call))
    } else if token(text, Kind::Dot)?.is_ok() {
        let func =
            not_application(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingUfc)?;
        let (arguments, end) = list(
            text,
            alloc,
            interner,
            Kind::LeftParen,
            Kind::RightParen,
            &mut expr,
            false,
        )?
        .map_err(Irrecoverable::WhileParsingUfc)?;

        let mut all_arguments = Vec::with_capacity(1 + arguments.len());
        all_arguments.push(first);
        all_arguments.extend(arguments);

        let span = first.span().union(&end);

        let call = Expr::Call {
            function: alloc.alloc(func),
            arguments: alloc.alloc_slice_fill_iter(all_arguments),
            span,
        };
        Ok(Ok(call))
    } else {
        Ok(Ok(first))
    }
}

fn field<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Field<'expr, &'ident str, &'ident str>> {
    let (name, start) = propagate!(identifier(text, interner));

    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingField)?;

    let value = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingField)?;

    Ok(Ok(Field {
        name,
        value,
        span: start.union(&value.span()),
    }))
}

fn branch<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Branch<'expr, &'ident str, &'ident str>> {
    let pattern = propagate!(pattern(text, alloc, interner));
    let _ = token(text, Kind::ThickArrow)?.map_err(Irrecoverable::WhileParsingBranch)?;
    let body = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingBranch)?;

    Ok(Ok(Branch {
        pattern,
        body,
        span: pattern.span().union(&body.span()),
    }))
}

fn case<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, &'ident str, &'ident str>> {
    let start = propagate!(token(text, Kind::Case));
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

fn r#let<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, &'ident str, &'ident str>> {
    let start = propagate!(token(text, Kind::Let));
    let left_side = pattern(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingLet)?;
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingLet)?;
    let right_side = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingLet)?;

    Ok(Ok(Statement::Let {
        left_side,
        right_side,
        span: right_side.span().union(&start.into()),
    }))
}

fn raw<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, &'ident str, &'ident str>> {
    let expr = propagate!(expr(text, alloc, interner));

    Ok(Ok(Statement::Raw(expr, expr.span())))
}

fn statement<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, &'ident str, &'ident str>> {
    or_try!(r#let(text, alloc, interner), r#raw(text, alloc, interner))
}

#[allow(clippy::missing_panics_doc)]
fn block<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, &'ident str, &'ident str>> {
    let start = propagate!(token(text, Kind::LeftBrace));
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
