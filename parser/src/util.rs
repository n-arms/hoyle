macro_rules! propogate {
    ($e:expr) => {
        match $e? {
            Ok(res) => res,
            Err(e) => return Ok(Err(e)),
        }
    };
}

pub(crate) use propogate;

use arena_alloc::{General, Interning, Specialized};
use ir::ast::Span;
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
    WhileParsingArrowType(Recoverable),
    WhileParsingList(Recoverable),
    WhileParsingField(Recoverable),
    WhileParsingUnionType(Recoverable),
    WhileParsingCase(Recoverable),
    WhileParsingBranch(Recoverable),
    WhileParsingPatternField(Recoverable),
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

pub fn list<'src, 'ident, 'expr, T, I>(
    text: &mut Peekable<I>,
    alloc: &General<'expr>,
    interner: &Interning<'ident, Specialized>,
    start_token: Kind,
    end_token: Kind,
    element: &mut impl FnMut(
        &mut Peekable<I>,
        &General<'expr>,
        &Interning<'ident, Specialized>,
    ) -> Result<T>,
    require_trailing_comma: bool,
) -> Result<(&'expr [T], Span)>
where
    I: Iterator<Item = Token<'src>> + Clone,
{
    let start = propogate!(token(text, start_token));
    let mut elements = Vec::new();
    let end;

    loop {
        if let Ok(end_span) = token(text, end_token)? {
            end = end_span;
            break;
        }
        let elem = element(text, alloc, interner)?.map_err(Irrecoverable::WhileParsingList)?;
        elements.push(elem);

        if let Err(error) = token(text, Kind::Comma)? {
            if require_trailing_comma {
                return Err(Irrecoverable::WhileParsingList(error));
            } else {
                match token(text, end_token)? {
                    Ok(end_span) => {
                        end = end_span;
                        break;
                    }
                    Err(error) => return Err(Irrecoverable::WhileParsingList(error)),
                }
            }
        }
    }

    let span = Span::from(start).union(&end.into());
    let element_slice = alloc.alloc_slice_fill_iter(elements);

    Ok(Ok((element_slice, span)))
}

#[allow(clippy::missing_panics_doc)]
pub fn token<'src>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
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

pub fn token_hint<'src>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    kind: Kind,
) -> bool {
    if let Some(token) = text.peek() {
        token.kind == kind
    } else {
        false
    }
}

pub fn identifier<'src, 'ident>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    interner: &Interning<'ident, Specialized>,
) -> Result<(&'ident str, Span)> {
    let span = propogate!(token(text, Kind::Identifier));
    Ok(Ok((interner.get_or_intern(span.data), span.into())))
}

#[allow(clippy::missing_panics_doc)]
pub fn variant_tag<'src, 'ident>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    interner: &Interning<'ident, Specialized>,
) -> Result<(&'ident str, Span)> {
    if let Some(token) = text.peek() {
        assert!(!token.span.data.is_empty());
        if token.kind == Kind::Identifier && token.span.data.chars().next().unwrap().is_uppercase()
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
