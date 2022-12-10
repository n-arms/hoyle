#[derive(Copy, Clone, Debug)]
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
