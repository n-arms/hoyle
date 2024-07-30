use core::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span<'a> {
    pub data: &'a str,
    pub offset: usize,
}

impl<'a> Span<'a> {
    #[must_use]
    pub const fn new(data: &'a str, offset: usize) -> Self {
        Span { data, offset }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Cross,
    Dash,
    Star,
    Slash,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Number,
    Identifier,
    UpperIdentifier,
    Func,
    Struct,
    Case,
    Of,
    Arrow,
    ThickArrow,
    Let,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSquareBracket,
    RightSquareBracket,
    Comma,
    Colon,
    Semicolon,
    SingleEquals,
    SingleBar,
    Dot,
    BinaryOperator(BinaryOperator),
}

#[derive(Clone, Default)]
pub struct List<'a> {
    kinds: Vec<Kind>,
    spans: Vec<Span<'a>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?} {}]", self.kind, self.span)
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for Span<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl fmt::Debug for List<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.clone().into_iter().map(|x| x))
            .finish()
    }
}
