macro_rules! propogate {
    ($e:expr) => {
        match $e? {
            Ok(res) => res,
            Err(e) => return Ok(Err(e)),
        }
    };
}

use arena_alloc::{General, Interning, Specialized};
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
    interner: &Interning<'ident, Specialized>,
) -> Result<(&'ident str, Span)> {
    let span = propogate!(token(text, Kind::Identifier));
    Ok(Ok((interner.get_or_intern(span.data), span.into())))
}

pub fn literal<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let span = propogate!(token(text, Kind::Number));
    Ok(Ok(Expr::Literal(
        Literal::Integer(alloc.alloc_str(span.data)),
        span.into(),
    )))
}

pub fn variable<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let (var, span) = propogate!(identifier(text, interner));
    Ok(Ok(Expr::Variable(var, span)))
}

pub fn parens<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let _ = propogate!(token(text, Kind::LeftParen));
    let expr = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingParens)?;
    let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingParens)?;
    Ok(Ok(expr))
}

pub fn not_application<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    or_try(
        variable(text, interner),
        or_try(
            literal(text, alloc),
            or_try(parens(text, alloc, interner), block(text, alloc, interner)),
        ),
    )
}

pub fn not_variant<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
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

pub fn variant_tag<'src, 'ident>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    interner: &Interning<'ident, Specialized>,
) -> Result<(&'ident str, Span)> {
    if let Some(token) = text.peek() {
        assert!(token.span.data.len() > 0);
        if token.kind == Kind::Identifier && token.span.data.chars().nth(0).unwrap().is_uppercase()
        {
            let token = text.next().unwrap();
            Ok(Ok((
                interner.get_or_intern(token.span.data),
                token.span.into(),
            )))
        } else {
            Ok(Err(Recoverable::Expected(
                vec![Kind::Identifier],
                Some(token.kind),
            )))
        }
    } else {
        Ok(Err(Recoverable::Expected(vec![Kind::Identifier], None)))
    }
}

pub fn variant<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let (variant, start) = propogate!(variant_tag(text, interner));

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
        variant,
        arguments: alloc.alloc_slice_fill_iter(args),
        span,
    }))
}

pub fn expr<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Expr<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    or_try(
        variant(text, alloc, interner),
        not_variant(text, alloc, interner),
    )
}

pub fn pattern<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    _alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Pattern<'expr, &'ident str>> {
    let (id, span) = propogate!(identifier(text, interner));
    Ok(Ok(Pattern::Variable(id, span)))
}

pub fn r#let<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
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
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let expr = propogate!(expr(text, alloc, interner));

    Ok(Ok(Statement::Raw(expr, expr.span())))
}

pub fn statement<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Statement<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    or_try(r#let(text, alloc, interner), r#raw(text, alloc, interner))
}

#[allow(clippy::missing_panics_doc)]
pub fn block<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
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

pub fn generic<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    _alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Generic<'ident>> {
    let (identifier, span) = propogate!(identifier(text, interner));
    Ok(Ok(Generic { identifier, span }))
}

pub fn r#type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    _alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let (name, span) = propogate!(identifier(text, interner));
    Ok(Ok(Type::Named(name, span)))
}

pub fn argument<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Argument<'expr, &'ident str, Type<'expr, 'ident>>> {
    let pattern = propogate!(pattern(text, alloc, interner));
    let _ = token(text, Kind::Colon)?.map_err(Irrecoverable::WhileParsingArgument)?;
    let type_annotation =
        r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingArgument)?;

    Ok(Ok(Argument {
        pattern,
        type_annotation,
        span: pattern.span().union(&type_annotation.span()),
    }))
}

pub fn generics<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<&'expr [Generic<'ident>]> {
    let _ = propogate!(token(text, Kind::LeftSquareBracket));
    if token(text, Kind::RightSquareBracket)?.is_ok() {
        return Ok(Ok(&[]));
    }
    let first = generic(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingGenerics)?;
    let mut rest = vec![first];

    loop {
        if token(text, Kind::Comma)?.is_err() {
            let _ = token(text, Kind::RightSquareBracket)?
                .map_err(Irrecoverable::WhileParsingGenerics)?;
            return Ok(Ok(alloc.alloc_slice_fill_iter(rest)));
        }
        rest.push(generic(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingGenerics)?);
    }
}

pub fn arguments<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<&'expr [Argument<'expr, &'ident str, Type<'expr, 'ident>>]> {
    let _ = propogate!(token(text, Kind::LeftParen));
    if token(text, Kind::RightParen)?.is_ok() {
        return Ok(Ok(&[]));
    }
    let first = argument(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingArguments)?;
    let mut rest = vec![first];

    loop {
        if token(text, Kind::Comma)?.is_err() {
            let _ = token(text, Kind::RightParen)?.map_err(Irrecoverable::WhileParsingArguments)?;
            return Ok(Ok(alloc.alloc_slice_fill_iter(rest)));
        }
        rest.push(argument(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingArguments)?);
    }
}

pub fn return_type<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Type<'expr, 'ident>> {
    let _ = propogate!(token(text, Kind::Colon));
    let r#type = r#type(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingReturnType)?;
    Ok(Ok(r#type))
}

pub fn definition<'src, 'ident, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>>>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Definition<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let start = propogate!(token(text, Kind::Func));
    let (name, _) = identifier(text, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let generics = generics(text, alloc, interner)?.unwrap_or_default();
    let arguments = arguments(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let return_type = return_type(text, alloc, interner)?.ok();
    let _ = token(text, Kind::SingleEquals)?.map_err(Irrecoverable::WhileParsingFunc)?;
    let body = expr(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingFunc)?;

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
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
) -> Result<Program<'expr, 'ident, &'ident str, Type<'expr, 'ident>>> {
    let mut defs = Vec::new();

    while text.peek().is_some() {
        let def = definition(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingProgram)?;
        defs.push(def);
    }

    Ok(Ok(Program {
        definitions: alloc.alloc_slice_fill_iter(defs),
    }))
}
