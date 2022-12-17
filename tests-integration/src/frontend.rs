use arena_alloc::*;
use bumpalo::Bump;
use ir::qualified::Program;
use lexer::scan_tokens;
use parser::parser;
use qualifier::definitions::Definitions;

fn run_frontend<'src, 'ident, 'qual>(
    text: &'src str,
    ident: &'ident Bump,
    qualified_tree: &'qual Bump,
) -> Program<'qual, 'ident> {
    let (tokens, errors) = scan_tokens(text);
    if !errors.success() {
        panic!("{:?}\n{:?}", tokens, errors);
    }
    let parse_tree = Bump::new();
    let ast_program = match parser::program(
        &mut tokens.into_iter().peekable(),
        &General::new(&parse_tree),
        &Interning::new(ident),
    ) {
        Ok(Ok(ast)) => ast,
        Err(error) => {
            panic!("{:?}\n{:?}", tokens, error);
        }
        Ok(Err(error)) => {
            panic!("{:?}\n{:?}", tokens, error);
        }
    };
    let mut defs = Definitions::default();
    let qualified_program = match qualifier::qualifier::program(
        ast_program,
        &mut defs,
        &Interning::new(ident),
        &General::new(qualified_tree),
    ) {
        Ok(prog) => prog,
        Err(error) => {
            panic!("{:?}\n{:?}\n{:?}", tokens, ast_program, error);
        }
    };
    qualified_program
}

#[test]
fn trivial_functions() {
    let ident = Bump::new();
    let qual = Bump::new();

    run_frontend("func f() = 0", &ident, &qual);
    run_frontend("func f[A](a: A): A = a", &ident, &qual);
    run_frontend("func f[A](a: A): A = {let x = a; x}", &ident, &qual);
}

#[test]
#[should_panic]
fn unqualified_variable() {
    let ident = Bump::new();
    let qual = Bump::new();

    run_frontend("func() = x", &ident, &qual);
}

#[test]
#[should_panic]
fn unqualified_type() {
    let ident = Bump::new();
    let qual = Bump::new();

    run_frontend("func(x: A) = x", &ident, &qual);
}
