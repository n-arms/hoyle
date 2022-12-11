macro_rules! try_unwrap {
    ($e:expr) => {
        if let Some(t) = $e {
            t
        } else {
            return Ok(Err(Recoverable::UnexpectedEof));
        }
    };
}

macro_rules! propogate {
    ($e:expr) => {
        match $e? {
            Ok(res) => res,
            Err(e) => return Ok(Err(e)),
        }
    };
}

use crate::alloc::*;
use ir::ast::*;
use ir::token::{self, Kind, Token};
use std::iter::Peekable;
use std::result;

#[derive(Debug)]
pub enum Recoverable {
    Expected(Vec<Kind>, Option<Kind>),
    UnexpectedEof,
}

impl Recoverable {
    fn combine(&self, other: Recoverable) -> Self {
        match (self, other) {
            (Self::Expected(wants1, got1), Self::Expected(mut wants2, got2)) => {
                assert_eq!(*got1, got2);

                wants2.extend(wants1);
                Self::Expected(wants2, got2)
            }
            (Self::UnexpectedEof, Self::UnexpectedEof) => Self::UnexpectedEof,
            (e1, e2) => panic!("{:?} {:?}", e1, e2),
        }
    }
}

#[derive(Debug)]
pub enum Irrecoverable {
    WhileParsingParens(Recoverable),
    WhileParsingLet(Recoverable),
    WhileParsingRaw(Recoverable),
    WhileParsingArgument(Recoverable),
    WhileParsingGenerics(Recoverable),
    WhileParsingFunc(Recoverable),
    WhileParsingArguments(Recoverable),
    WhileParsingReturnType(Recoverable),
    WhileParsingProgram(Recoverable),
}

pub type Result<T> = result::Result<result::Result<T, Recoverable>, Irrecoverable>;
pub type Parser<'src, 'ident, 'expr, T, I> =
    fn(text: &mut Peekable<I>, alloc: General<'ident, 'expr>) -> Result<T>;

pub fn or_try<T>(left: Result<T>, right: Result<T>) -> Result<T> {
    match left? {
        Ok(result) => Ok(Ok(result)),
        Err(error) => match right? {
            Ok(result) => Ok(Ok(result)),
            Err(error2) => Ok(Err(error.combine(error2))),
        },
    }
}

pub fn token<'src>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    kind: Kind,
) -> Result<token::Span<'src>> {
    let token = try_unwrap!(text.peek());
    if token.kind == kind {
        Ok(Ok(text.next().unwrap().span))
    } else {
        Ok(Err(Recoverable::Expected(vec![kind], Some(token.kind))))
    }
}

pub fn identifier<'src, 'ident>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, '_>,
) -> Result<(&'ident str, Span)> {
    let span = propogate!(token(text, Kind::Identifier));
    Ok(Ok((alloc.get_or_intern(span.data), span.into())))
}

pub fn literal<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let span = propogate!(token(text, Kind::Number));
    Ok(Ok(Expr::Literal(
        Literal::Integer(alloc.ast_alloc_str(span.data)),
        span.into(),
    )))
}

pub fn variable<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let (var, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Expr::Variable(var, span)))
}

pub fn parens<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let _ = propogate!(token(text, Kind::LeftParen));
    let expr = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingParens)?;
    let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingParens)?;
    Ok(Ok(expr))
}

pub fn not_application<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    or_try(
        variable(text, alloc.clone()),
        or_try(literal(text, alloc.clone()), parens(text, alloc.clone())),
    )
}

pub fn expr<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let func = propogate!(not_application(text, alloc.clone()));
    let mut args = Vec::new();

    while let Ok(next) = not_application(text, alloc.clone())? {
        args.push(next);
    }

    if args.is_empty() {
        Ok(Ok(func))
    } else {
        Ok(Ok(Expr::Call {
            function: alloc.ast_alloc(func),
            arguments: alloc.ast_alloc_slice_copy(&args),
            span: func.span().union(&args.last().unwrap().span()),
        }))
    }
}

pub fn pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Pattern<'ident, 'expr>> {
    let (id, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Pattern::Variable(id, span.into())))
}

pub fn r#let<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Statement<'ident, 'expr>> {
    let start = propogate!(token(text, Kind::Let));
    let left_side = pattern(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingLet)?;
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingLet)?;
    let right_side = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingLet)?;
    let end = token(text, Kind::Semicolon)?.map_err(Irrecoverable::WhileParsingLet)?;

    Ok(Ok(Statement::Let {
        left_side,
        right_side,
        span: Span::from(start).union(&end.into()),
    }))
}

pub fn raw<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Statement<'ident, 'expr>> {
    let expr = propogate!(expr(text, alloc.clone()));
    let end = token(text, Kind::Semicolon)?.map_err(Irrecoverable::WhileParsingRaw)?;

    Ok(Ok(Statement::Raw(expr, expr.span().union(&end.into()))))
}

pub fn statement<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Statement<'ident, 'expr>> {
    or_try(r#let(text, alloc.clone()), r#raw(text, alloc))
}

pub fn generic<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Generic<'ident>> {
    let (identifier, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Generic { identifier, span }))
}

pub fn r#type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Type<'ident, 'expr>> {
    let (name, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Type::Named(name, span)))
}

pub fn argument<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Argument<'ident, 'expr>> {
    let pattern = propogate!(pattern(text, alloc.clone()));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingArgument)?;
    let type_annotation = r#type(text, alloc)?.map_err(Irrecoverable::WhileParsingArgument)?;

    Ok(Ok(Argument {
        pattern,
        type_annotation,
        span: pattern.span().union(&type_annotation.span()),
    }))
}

pub fn generics<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<&'expr [Generic<'ident>]> {
    let _ = propogate!(token(text, Kind::LeftSquareBracket));
    if let Ok(_) = token(text, Kind::RightSquareBracket)? {
        return Ok(Ok(&[]));
    }
    let first = generic(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingGenerics)?;
    let mut rest = vec![first];

    loop {
        if let Err(_) = token(text, Kind::Comma)? {
            let _ = token(text, Kind::RightSquareBracket)?
                .map_err(Irrecoverable::WhileParsingGenerics)?;
            return Ok(Ok(alloc.ast_alloc_slice_copy(&rest)));
        }
        rest.push(generic(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingGenerics)?);
    }
}

pub fn arguments<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<&'expr [Argument<'ident, 'expr>]> {
    let _ = propogate!(token(text, Kind::LeftParen));
    if let Ok(_) = token(text, Kind::RightParen)? {
        return Ok(Ok(&[]));
    }
    let first = argument(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingArguments)?;
    let mut rest = vec![first];

    loop {
        if let Err(_) = token(text, Kind::Comma)? {
            let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingArguments)?;
            return Ok(Ok(alloc.ast_alloc_slice_copy(&rest)));
        }
        rest.push(argument(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingArguments)?);
    }
}

pub fn return_type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Type<'ident, 'expr>> {
    let _ = propogate!(token(text, Kind::Colon));
    let r#type = r#type(text, alloc)?.map_err(Irrecoverable::WhileParsingReturnType)?;
    Ok(Ok(r#type))
}

pub fn definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Definition<'ident, 'expr>> {
    let start = propogate!(token(text, Kind::Func));
    let (name, _) = identifier(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingFunc)?;
    let generics = generics(text, alloc.clone())?.unwrap_or_default();
    let arguments = arguments(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingFunc)?;
    let return_type = return_type(text, alloc.clone())?.ok();
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let body = expr(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingFunc)?;

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
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: General<'ident, 'expr>,
) -> Result<Program<'ident, 'expr>> {
    let mut defs = Vec::new();

    while let Some(_) = text.peek() {
        let def = definition(text, alloc.clone())?.map_err(Irrecoverable::WhileParsingProgram)?;
        defs.push(def);
    }

    Ok(Ok(Program {
        definitions: alloc.ast_alloc_slice_copy(&defs),
    }))
}
