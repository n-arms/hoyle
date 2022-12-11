macro_rules! propogate {
    ($e:expr) => {
        match $e? {
            Ok(res) => res,
            Err(e) => return Ok(Err(e)),
        }
    };
}

use crate::alloc::General;
use ir::ast::{
    Argument, Block, Definition, Expr, Generic, Literal, Pattern, Program, Span, Statement, Type,
};
use ir::token::{self, Kind, Token};
use std::iter::Peekable;
use std::result;

#[derive(Debug)]
pub enum Recoverable {
    Expected(Vec<Kind>, Option<Kind>),
}

impl Recoverable {
    fn combine(&self, other: Self) -> Self {
        match (self, other) {
            (Self::Expected(wants1, got1), Self::Expected(mut wants2, got2)) => {
                assert_eq!(*got1, got2);

                wants2.extend(wants1);
                Self::Expected(wants2, got2)
            }
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
    WhileParsingBlock(Recoverable),
    MissingSemicolon(Span),
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

#[allow(clippy::missing_panics_doc)]
pub fn token<'src>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    kind: Kind,
) -> Result<token::Span<'src>> {
    let token = if let Some(token) = text.peek() {
        token
    } else {
        return Ok(Err(Recoverable::Expected(vec![kind], None)));
    };
    if token.kind == kind {
        Ok(Ok(text.next().unwrap().span))
    } else {
        Ok(Err(Recoverable::Expected(vec![kind], Some(token.kind))))
    }
}

pub fn identifier<'src, 'ident>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, '_>,
) -> Result<(&'ident str, Span)> {
    let span = propogate!(token(text, Kind::Identifier));
    Ok(Ok((alloc.get_or_intern(span.data), span.into())))
}

pub fn literal<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let span = propogate!(token(text, Kind::Number));
    Ok(Ok(Expr::Literal(
        Literal::Integer(alloc.ast_alloc_str(span.data)),
        span.into(),
    )))
}

pub fn variable<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let (var, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Expr::Variable(var, span)))
}

pub fn parens<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let _ = propogate!(token(text, Kind::LeftParen));
    let expr = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingParens)?;
    let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingParens)?;
    Ok(Ok(expr))
}

pub fn not_application<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    or_try(
        variable(text, alloc),
        or_try(
            literal(text, alloc),
            or_try(parens(text, alloc), block(text, alloc)),
        ),
    )
}

pub fn expr<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let func = propogate!(not_application(text, alloc));
    let mut args = Vec::new();

    while let Ok(next) = not_application(text, alloc)? {
        args.push(next);
    }

    if let Some(last) = args.last() {
        Ok(Ok(Expr::Call {
            function: alloc.ast_alloc(func),
            arguments: alloc.ast_alloc_slice_copy(&args),
            span: func.span().union(&last.span()),
        }))
    } else {
        Ok(Ok(func))
    }
}

pub fn pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Pattern<'ident, 'expr>> {
    let (id, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Pattern::Variable(id, span)))
}

pub fn r#let<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Statement<'ident, 'expr>> {
    let start = propogate!(token(text, Kind::Let));
    let left_side = pattern(text, alloc)?.map_err(Irrecoverable::WhileParsingLet)?;
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingLet)?;
    let right_side = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingLet)?;

    Ok(Ok(Statement::Let {
        left_side,
        right_side,
        span: right_side.span().union(&start.into()),
    }))
}

pub fn raw<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Statement<'ident, 'expr>> {
    let expr = propogate!(expr(text, alloc));

    Ok(Ok(Statement::Raw(expr, expr.span())))
}

pub fn statement<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Statement<'ident, 'expr>> {
    or_try(r#let(text, alloc), r#raw(text, alloc))
}

#[allow(clippy::missing_panics_doc)]
pub fn block<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Expr<'ident, 'expr>> {
    let start = propogate!(token(text, Kind::LeftBrace));
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
                statements: alloc.ast_alloc_slice_copy(&rest),
                result: Some(alloc.ast_alloc(result)),
                span: Span::from(start).union(&end.into()),
            })));
        }
        if let Ok(end) = token(text, Kind::RightBrace)? {
            return Ok(Ok(Expr::Block(Block {
                statements: alloc.ast_alloc_slice_copy(&rest),
                result: None,
                span: Span::from(start).union(&end.into()),
            })));
        }
        let stmt = statement(text, alloc)?.map_err(Irrecoverable::WhileParsingBlock)?;
        rest.push(stmt);
    }
}

pub fn generic<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Generic<'ident>> {
    let (identifier, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Generic { identifier, span }))
}

pub fn r#type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Type<'ident, 'expr>> {
    let (name, span) = propogate!(identifier(text, alloc));
    Ok(Ok(Type::Named(name, span)))
}

pub fn argument<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Argument<'ident, 'expr>> {
    let pattern = propogate!(pattern(text, alloc));
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
    alloc: &mut General<'ident, 'expr>,
) -> Result<&'expr [Generic<'ident>]> {
    let _ = propogate!(token(text, Kind::LeftSquareBracket));
    if token(text, Kind::RightSquareBracket)?.is_ok() {
        return Ok(Ok(&[]));
    }
    let first = generic(text, alloc)?.map_err(Irrecoverable::WhileParsingGenerics)?;
    let mut rest = vec![first];

    loop {
        if token(text, Kind::Comma)?.is_err() {
            let _ = token(text, Kind::RightSquareBracket)?
                .map_err(Irrecoverable::WhileParsingGenerics)?;
            return Ok(Ok(alloc.ast_alloc_slice_copy(&rest)));
        }
        rest.push(generic(text, alloc)?.map_err(Irrecoverable::WhileParsingGenerics)?);
    }
}

pub fn arguments<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<&'expr [Argument<'ident, 'expr>]> {
    let _ = propogate!(token(text, Kind::LeftParen));
    if token(text, Kind::RightParen)?.is_ok() {
        return Ok(Ok(&[]));
    }
    let first = argument(text, alloc)?.map_err(Irrecoverable::WhileParsingArguments)?;
    let mut rest = vec![first];

    loop {
        if token(text, Kind::Comma)?.is_err() {
            let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingArguments)?;
            return Ok(Ok(alloc.ast_alloc_slice_copy(&rest)));
        }
        rest.push(argument(text, alloc)?.map_err(Irrecoverable::WhileParsingArguments)?);
    }
}

pub fn return_type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Type<'ident, 'expr>> {
    let _ = propogate!(token(text, Kind::Colon));
    let r#type = r#type(text, alloc)?.map_err(Irrecoverable::WhileParsingReturnType)?;
    Ok(Ok(r#type))
}

pub fn definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &mut General<'ident, 'expr>,
) -> Result<Definition<'ident, 'expr>> {
    let start = propogate!(token(text, Kind::Func));
    let (name, _) = identifier(text, alloc)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let generics = generics(text, alloc)?.unwrap_or_default();
    let arguments = arguments(text, alloc)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let return_type = return_type(text, alloc)?.ok();
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let body = expr(text, alloc)?.map_err(Irrecoverable::WhileParsingFunc)?;

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
    alloc: &mut General<'ident, 'expr>,
) -> Result<Program<'ident, 'expr>> {
    let mut defs = Vec::new();

    while text.peek().is_some() {
        let def = definition(text, alloc)?.map_err(Irrecoverable::WhileParsingProgram)?;
        defs.push(def);
    }

    Ok(Ok(Program {
        definitions: alloc.ast_alloc_slice_copy(&defs),
    }))
}
