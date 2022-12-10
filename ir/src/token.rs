use crate::span::Span;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Cross,
    Dash,
    Star,
    Slash,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Number,
    Identifier,
    Func,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSquareBracket,
    RightSquareBracket,
    Comma,
    Colon,
    BinaryOperator(BinaryOperator),
}

#[derive(Debug, Default)]
pub struct List<'a> {
    kinds: Vec<Kind>,
    spans: Vec<Span<'a>>,
}

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    pub kind: Kind,
    pub span: Span<'a>,
}

impl<'a> List<'a> {
    pub fn push(&mut self, kind: Kind, span: Span<'a>) {
        assert_eq!(self.kinds.len(), self.spans.len());
        self.kinds.push(kind);
        self.spans.push(span);
    }
}

#[derive(Clone)]
pub struct Tokens<'a> {
    tokens: &'a List<'a>,
    offset: usize,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = Token {
            kind: *self.tokens.kinds.get(self.offset)?,
            span: *self.tokens.spans.get(self.offset)?,
        };

        self.offset += 1;

        Some(token)
    }
}

impl<'a> IntoIterator for &'a List<'a> {
    type Item = Token<'a>;
    type IntoIter = Tokens<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Tokens {
            tokens: self,
            offset: 0,
        }
    }
}
