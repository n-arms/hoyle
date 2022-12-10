#[derive(Copy, Clone, Debug)]
pub struct Span<'a> {
    data: &'a str,
    offset: usize,
}
