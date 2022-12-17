use arena_alloc::*;
use bumpalo::Bump;
use ir::qualified::Program;
use lexer::scan_tokens;
use parser::parser;
use qualifier::qualifier;

fn run_frontend<'src, 'ident, 'parse, 'qual>(
    text: &'src str,
    ident: &'ident Bump,
    parse_tree: &'parse Bump,
    qualified_tree: &'qual Bump,
) -> Program<'ident, 'qual> {
    let tokens = scan_tokens(text);
    todo!()
}

#[test]
fn trivial_functions() {
    panic!()
}
