use tree::token::Span;

#[derive(Debug)]
pub struct SpanSource<'a> {
    data: &'a str,
}

#[allow(clippy::expect_used)]
impl<'a> SpanSource<'a> {
    #[must_use]
    pub fn span(&self, start: usize, end: usize) -> Span<'a> {
        //dbg!(len, self.data.chars().skip(start).collect::<Vec<_>>());
        let data = self
            .data
            .get(start..end)
            .expect("Out of bounds span creation");
        Span::new(data, start)
    }

    #[must_use]
    pub const fn new(data: &'a str) -> Self {
        SpanSource { data }
    }
}
