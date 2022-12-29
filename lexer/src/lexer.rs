use crate::span_source::SpanSource;
use ir::token::{self, BinaryOperator, Kind};

#[derive(Debug, Default)]
pub struct Errors {
    errors: Vec<(usize, char)>,
}

impl Errors {
    pub fn unknown_character(&mut self, idx: usize, char: char) {
        self.errors.push((idx, char));
    }

    #[must_use]
    pub fn success(&self) -> bool {
        self.errors.is_empty()
    }
}

#[must_use]
pub fn scan_tokens(text: &str) -> (token::List, Errors) {
    let mut chars = text.char_indices().peekable();
    let mut tokens = token::List::default();
    let source = SpanSource::new(text);
    let mut errors = Errors::default();

    while let Some((start, char)) = chars.next() {
        let kind = match char {
            '(' => Kind::LeftParen,
            ')' => Kind::RightParen,
            '[' => Kind::LeftSquareBracket,
            ']' => Kind::RightSquareBracket,
            '{' => Kind::LeftBrace,
            '}' => Kind::RightBrace,
            '+' => Kind::BinaryOperator(BinaryOperator::Cross),
            '-' => Kind::BinaryOperator(BinaryOperator::Dash),
            '*' => Kind::BinaryOperator(BinaryOperator::Star),
            '/' => Kind::BinaryOperator(BinaryOperator::Slash),
            ',' => Kind::Comma,
            ':' => Kind::Colon,
            ';' => Kind::Semicolon,
            '.' => Kind::Dot,
            '=' if matches!(chars.peek(), Some((_, '>'))) => {
                chars.next();
                Kind::ThickArrow
            }
            '=' => Kind::SingleEquals,
            '|' => Kind::SingleBar,
            w if w.is_whitespace() => continue,
            n if n.is_numeric() => {
                let mut end = None;
                while let Some((idx, char)) = chars.peek() {
                    if char.is_numeric() {
                        let _ = chars.next();
                    } else {
                        end = Some(*idx);
                        break;
                    }
                }
                let span = source.span(start, end.unwrap_or(text.len()));
                let kind = Kind::Number;
                tokens.push(kind, span);
                continue;
            }
            c if c.is_alphabetic() => {
                let mut end = None;
                while let Some((idx, char)) = chars.peek() {
                    if !char.is_alphanumeric() && *char != '_' {
                        end = Some(*idx);
                        break;
                    } else {
                        let _ = chars.next();
                    }
                }
                let span = source.span(start, end.unwrap_or(text.len()));
                let kind = match span.data {
                    "func" => Kind::Func,
                    "struct" => Kind::Struct,
                    "let" => Kind::Let,
                    "case" => Kind::Case,
                    "of" => Kind::Of,
                    _ => Kind::Identifier,
                };
                tokens.push(kind, span);
                continue;
            }
            _ => {
                errors.unknown_character(start, char);
                continue;
            }
        };
        let end = if let Some((idx, _)) = chars.peek() {
            *idx
        } else {
            text.len()
        };
        let span = source.span(start, end);
        tokens.push(kind, span);
    }

    (tokens, errors)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokens() {
        let text = "123abc([{}])+- */funca3_4:,func";
        let (tokens, errors) = scan_tokens(text);
        assert!(errors.success());

        let kinds = [
            Kind::Number,
            Kind::Identifier,
            Kind::LeftParen,
            Kind::LeftSquareBracket,
            Kind::LeftBrace,
            Kind::RightBrace,
            Kind::RightSquareBracket,
            Kind::RightParen,
            Kind::BinaryOperator(BinaryOperator::Cross),
            Kind::BinaryOperator(BinaryOperator::Dash),
            Kind::BinaryOperator(BinaryOperator::Star),
            Kind::BinaryOperator(BinaryOperator::Slash),
            Kind::Identifier,
            Kind::Colon,
            Kind::Comma,
            Kind::Func,
        ];

        for (token, kind) in tokens.into_iter().zip(kinds) {
            assert_eq!(token.kind, kind);
        }
    }
}
