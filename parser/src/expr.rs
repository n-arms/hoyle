use crate::pattern::pattern;
use crate::util::{identifier, list, or_try, propagate, token, Irrecoverable, Recoverable, Result};
use arena_alloc::{General, Interning, Specialized};
use ir::source::{Block, Branch, Expr, Field, Literal, Span, Statement};
use ir::token::{Kind, Token};
use smartstring::{LazyCompact, SmartString};
use std::iter::Peekable;

fn literal<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr>> {
    let span = propagate!(token(text, Kind::Number));
    Ok(Ok(Expr::Literal {
        literal: Literal::Number(propagate!(Ok(span
            .data
            .parse()
            .map_err(|_| Recoverable::Expected(vec![Kind::Number], None))))),
        span: span.into(),
    }))
}

fn parens<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr>> {
    let _ = propagate!(token(text, Kind::LeftParen));
    let expr = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingParens)?;
    let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingParens)?;
    Ok(Ok(expr))
}

pub fn struct_or_variable<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr>> {
    let (identifier, start) = propagate!(identifier(text));

    let struct_list = list(
        text,
        alloc,
        Kind::LeftBrace,
        Kind::RightBrace,
        &mut field,
        false,
    )?;

    if let Ok((fields, end)) = struct_list {
        Ok(Ok(Expr::StructLiteral {
            name: identifier,
            fields,
            span: end.union(&start),
        }))
    } else {
        Ok(Ok(Expr::Variable {
            identifier,
            span: start,
        }))
    }
}

pub fn not_application<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr>> {
    or_try!(
        block(text, alloc),
        literal(text, alloc),
        case(text, alloc),
        parens(text, alloc),
        struct_or_variable(text, alloc)
    )
}

pub fn expr<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr>> {
    let first = propagate!(not_application(text, alloc));
    let argument_list = list(
        text,
        alloc,
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
        let func = not_application(text, alloc)?.map_err(Irrecoverable::WhileParsingUfc)?;
        let (arguments, end) = list(
            text,
            alloc,
            Kind::LeftParen,
            Kind::RightParen,
            &mut expr,
            false,
        )?
        .map_err(Irrecoverable::WhileParsingUfc)?;

        let span = first.span().union(&end);

        let mut all_arguments = Vec::with_capacity(1 + arguments.len());
        all_arguments.push(first);
        all_arguments.extend(arguments.into_iter().cloned());

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

fn field<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Field<'expr>> {
    let (name, start) = propagate!(identifier(text));

    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingField)?;

    let value = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingField)?;

    Ok(Ok(Field { name, value }))
}

fn branch<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Branch<'expr>> {
    let pattern = propagate!(pattern(text, alloc));
    let _ = token(text, Kind::ThickArrow)?.map_err(Irrecoverable::WhileParsingBranch)?;
    let body = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingBranch)?;

    Ok(Ok(Branch {
        span: pattern.span().union(&body.span()),
        pattern,
        body,
    }))
}

fn case<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr>> {
    let start = propagate!(token(text, Kind::Case));
    let predicate = alloc.alloc(expr(text, alloc)?.map_err(Irrecoverable::WhileParsingCase)?);
    let _ = token(text, Kind::Of)?.map_err(Irrecoverable::WhileParsingCase)?;
    let (branches, end) = list(
        text,
        alloc,
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

fn r#let<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Statement<'expr>> {
    let start = propagate!(token(text, Kind::Let));
    let pattern = pattern(text, alloc)?.map_err(Irrecoverable::WhileParsingLet)?;
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingLet)?;
    let value = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingLet)?;

    Ok(Ok(Statement::Let {
        span: value.span().union(&start.into()),
        pattern,
        value,
    }))
}

fn raw<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Statement<'expr>> {
    let expr = propagate!(expr(text, alloc));

    Ok(Ok(Statement::Raw(expr)))
}

fn statement<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Statement<'expr>> {
    or_try!(r#let(text, alloc), r#raw(text, alloc))
}

#[allow(clippy::missing_panics_doc)]
fn block<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr>> {
    let start = propagate!(token(text, Kind::LeftBrace));
    if let Ok(end) = token(text, Kind::RightBrace)? {
        return Ok(Ok(Expr::Block(Block {
            statements: &[],
            result: None,
            span: Span::from(start).union(&end.into()),
        })));
    }
    let first = statement(text, alloc)?.map_err(Irrecoverable::WhileParsingBlock)?;
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
        let stmt = statement(text, alloc)?.map_err(Irrecoverable::WhileParsingBlock)?;
        rest.push(stmt);
    }
}
