use crate::span::Span;

#[derive(Copy, Clone, Debug)]
pub enum BinaryOperator {
    Cross,
    Dash,
    Star,
    Slash,
}

#[derive(Copy, Clone, Debug)]
pub enum TokenKind {
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
    BinaryOperator(BinaryOperator),
    Variant,
}

#[derive(Debug, Default)]
pub struct TokenList<'a> {
    kinds: Vec<TokenKind>,
    spans: Vec<Span<'a>>,
}

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    kind: TokenKind,
    span: Span<'a>,
}

impl<'a> TokenList<'a> {
    pub fn push(&mut self, kind: TokenKind, span: Span<'a>) {
        assert_eq!(self.kinds.len(), self.spans.len());
        self.kinds.push(kind);
        self.spans.push(span);
    }
}

#[derive(Copy, Clone)]
pub struct Tokens<'a> {
    tokens: &'a TokenList<'a>,
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
