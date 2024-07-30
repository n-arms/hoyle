use chumsky::recursive::recursive;
use chumsky::{error::Simple, primitive::filter_map, Parser};
use tree::parsed::*;
use tree::sized::Primitive;
use tree::token::{BinaryOperator, Kind, Token};
use tree::String;

pub fn token<'src>(kind: Kind) -> parser!('src, Token<'src>) {
    filter_map(move |span, t: Token| {
        if t.kind == kind {
            Ok(t)
        } else {
            Err(Simple::custom(span, format!("{} is not a {:?}", t, kind)))
        }
    })
}

pub fn token_text<'src>(kind: Kind) -> parser!('src, String) {
    token(kind).map(|token| String::from(token.span.data))
}

pub fn program<'src>() -> parser!('src, Program) {
    struct_definition()
        .or(function_definition())
        .repeated()
        .map(|defs| Program {
            structs: defs
                .clone()
                .into_iter()
                .flat_map(|def| def.structs)
                .collect(),
            functions: defs.into_iter().flat_map(|def| def.functions).collect(),
        })
}

fn named_type<'src>() -> parser!('src, String) {
    token_text(Kind::UpperIdentifier)
}

fn ident<'src>() -> parser!('src, String) {
    token_text(Kind::Identifier)
}

fn struct_definition<'src>() -> parser!('src, Program) {
    token(Kind::Struct)
        .ignore_then(named_type())
        .then_ignore(token(Kind::LeftBrace))
        .then(field_definition().repeated())
        .then_ignore(token(Kind::RightBrace))
        .map(|(name, fields)| Struct { name, fields })
        .map(Program::from_struct)
}

fn field_definition<'src>() -> parser!('src, Field) {
    ident()
        .then_ignore(token(Kind::Colon))
        .then(typ())
        .map(|(name, typ)| Field { name, typ })
}

fn typ<'src>() -> parser!('src, Type) {
    recursive(|typ| {
        named_type()
            .map(|name| Type::Named {
                name,
                arguments: Vec::new(),
            })
            .or(ident().map(|name| Type::Generic { name }))
            .or(typ
                .clone()
                .then_ignore(token(Kind::Arrow))
                .then(typ.clone())
                .map(|(arg, result)| Type::Function {
                    arguments: vec![arg],
                    result: Box::new(result),
                }))
            .or(token(Kind::LeftParen)
                .ignore_then(typ.clone().separated_by(token(Kind::Comma)))
                .then_ignore(token(Kind::RightParen))
                .then_ignore(token(Kind::Arrow))
                .then(typ)
                .map(|(arguments, result)| Type::Function {
                    arguments,
                    result: Box::new(result),
                }))
    })
}

fn argument<'src>() -> parser!('src, Argument) {
    ident()
        .then_ignore(token(Kind::Colon))
        .then(typ())
        .map(|(name, typ)| Argument { name, typ })
}

fn function_definition<'src>() -> parser!('src, Program) {
    let generic_list = token(Kind::LeftSquareBracket)
        .ignore_then(
            ident()
                .map(|name| Generic { name })
                .separated_by(token(Kind::Comma)),
        )
        .then_ignore(token(Kind::RightSquareBracket))
        .or_not()
        .map(|list| list.unwrap_or(Vec::new()));
    let argument_list = token(Kind::LeftParen)
        .ignore_then(argument().separated_by(token(Kind::Comma)))
        .then_ignore(token(Kind::RightParen));
    token(Kind::Func)
        .ignore_then(ident())
        .then(generic_list)
        .then(argument_list)
        .then_ignore(token(Kind::Colon))
        .then(typ())
        .then_ignore(token(Kind::SingleEquals))
        .then(expr())
        .map(|((((name, generics), arguments), result), body)| Function {
            name,
            generics,
            arguments,
            result,
            body,
        })
        .map(Program::from_function)
}

trait WithOperation<'src>: Parser<Token<'src>, Expr, Error = Simple<Token<'src>>> + Clone {
    fn with_operation<T>(
        self,
        symbol: parser!('src, T),
        operation: impl Fn(Expr, Expr) -> Expr + Clone,
    ) -> parser!('src, Expr) {
        self.clone()
            .then(symbol.ignore_then(self).repeated())
            .foldl(operation)
    }
}

impl<'src, T> WithOperation<'src> for T where
    T: Parser<Token<'src>, Expr, Error = Simple<Token<'src>>> + Clone
{
}

pub fn expr<'src>() -> parser!('src, Expr) {
    recursive(|e| {
        terminal(e)
            .with_operation(token(Kind::BinaryOperator(BinaryOperator::Star)), |a, b| {
                Expr::Primitive {
                    primitive: Primitive::Mul,
                    arguments: vec![a, b],
                }
            })
            .with_operation(
                token(Kind::BinaryOperator(BinaryOperator::Cross)),
                |a, b| Expr::Primitive {
                    primitive: Primitive::Add,
                    arguments: vec![a, b],
                },
            )
            .with_operation(token(Kind::BinaryOperator(BinaryOperator::Dash)), |a, b| {
                Expr::Primitive {
                    primitive: Primitive::Sub,
                    arguments: vec![a, b],
                }
            })
    })
}

fn comma_list<'src, T>(element: parser!('src, T)) -> parser!('src, Vec<T>) {
    element
        .clone()
        .then(token(Kind::Comma).ignore_then(element).repeated())
        .map(|(first, mut rest)| {
            rest.insert(0, first);
            rest
        })
        .or_not()
        .map(|list| list.unwrap_or_default())
}

fn pack_field<'src>(expr: parser!('src, Expr)) -> parser!('src, PackField) {
    ident()
        .then_ignore(token(Kind::Colon))
        .then(expr)
        .map(|(name, value)| PackField { name, value })
}

fn struct_pack<'src>(expr: parser!('src, Expr)) -> parser!('src, Expr) {
    named_type()
        .then_ignore(token(Kind::LeftBrace))
        .then(comma_list(pack_field(expr.clone())))
        .then_ignore(token(Kind::RightBrace))
        .map(|(name, fields)| Expr::StructPack {
            name,
            fields,
            tag: (),
        })
}

fn terminal<'src>(expr: parser!('src, Expr)) -> parser!('src, Expr) {
    literal_expr()
        .or(boolean_literal())
        .or(struct_pack(expr.clone()))
        .or(ident()
            .then_ignore(token(Kind::LeftParen))
            .then(comma_list(expr.clone()))
            .then_ignore(token(Kind::RightParen))
            .map(|(function, arguments)| Expr::CallDirect {
                function,
                arguments,
                tag: (),
            }))
        .or(ident().map(|name| Expr::Variable { name, typ: () }))
        .or(block(expr).map(|block| Expr::Block(block)))
}

fn boolean_literal<'src>() -> parser!('src, Expr) {
    token(Kind::True)
        .map(|_| Literal::Boolean(true))
        .or(token(Kind::False).map(|_| Literal::Boolean(false)))
        .map(|literal| Expr::Literal { literal })
}

fn literal_expr<'src>() -> parser!('src, Expr) {
    token_text(Kind::Number).try_map(|text, span| {
        text.parse::<f64>()
            .map_err(|err| Simple::custom(span, format!("{} is not a float: {:?}", text, err)))
            .map(|float| Expr::Literal {
                literal: Literal::Float(float),
            })
    })
}

fn block<'src>(expr: parser!('src, Expr)) -> parser!('src, Block) {
    token(Kind::LeftBrace)
        .ignore_then(
            statement(expr.clone())
                .then_ignore(token(Kind::Semicolon))
                .repeated(),
        )
        .then(expr)
        .map(|(stmts, result)| Block {
            stmts,
            result: Box::new(result),
        })
        .then_ignore(token(Kind::RightBrace))
}

fn statement<'src>(expr: parser!('src, Expr)) -> parser!('src, Statement) {
    token(Kind::Let)
        .ignore_then(ident())
        .then_ignore(token(Kind::SingleEquals))
        .then(expr)
        .map(|(name, value)| Statement::Let {
            name,
            typ: (),
            value,
        })
}
