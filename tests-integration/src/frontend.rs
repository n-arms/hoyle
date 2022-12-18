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
    run_frontend("func f[t](a: t): t = a", &ident, &qual);
    run_frontend("func f[t](a: t): t = {let x = a; x}", &ident, &qual);
    run_frontend("func f[]() = Ok 5", &ident, &qual);
    run_frontend("func f[a](x: a): V a = V x", &ident, &qual);
    run_frontend(
        "func call[a, b](f: func(a): b, x: a): b = f x",
        &ident,
        &qual,
    );
    run_frontend("func f() = {x: 5,}", &ident, &qual);
    run_frontend("func wrap[a](x: a): {x: a,} = {x: x,}", &ident, &qual);
    run_frontend("func f[a](x: a): V a | U a = V x", &ident, &qual);
    run_frontend("func add(x: int, y: int): int = 5", &ident, &qual);
    run_frontend(
        "func if[a](predicate: bool, branch_if: a, branch_else: a): a = branch_if",
        &ident,
        &qual,
    );
    run_frontend(
        "func unwrap[a](value: V a | U a): a = case value of {V unwrapped => unwrapped, U unwrapped => unwrapped}", 
        &ident, 
        &qual
    );
    run_frontend(
        "func unbox[a]({inner: boxed,}: {inner: a,}): a = boxed",
        &ident,
        &qual
    );
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

#[test]
#[should_panic]
fn inappropriate_variable() {
    let ident = Bump::new();
    let qual = Bump::new();

    run_frontend("func[a](x: a): x = x", &ident, &qual);
}

#[test]
#[should_panic]
fn inappropriate_type() {
    let ident = Bump::new();
    let qual = Bump::new();

    run_frontend("func[a](x: a): a = a", &ident, &qual);
}

#[test]
#[should_panic]
fn missing_trailing_record_comma() {
    let ident = Bump::new();
    let qual = Bump::new();

    run_frontend("func() = {x: 5}", &ident, &qual);
}

#[test]
#[should_panic]
fn unqualified_record_variable() {
    let ident = Bump::new();
    let qual = Bump::new();

    run_frontend("func() = {x: x,}", &ident, &qual);
}
