use ir::span::Span;

#[derive(Debug, Default)]
pub struct SpanSource<'a> {
    data: &'a str,
}

#[allow(clippy::expect_used)]
impl<'a> SpanSource<'a> {
    #[must_use]
    pub fn span(&self, start: usize, len: usize) -> Span<'a> {
        let end = start
            .checked_add(len)
            .expect("Hoyle doesn't support lexing large strings");
        let data = self
            .data
            .get(start..end)
            .expect("Out of bounds span creation");
        Span::new(data, start)
    }
}
