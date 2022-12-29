use arena_alloc::*;
use bumpalo::Bump;
use lexer::scan_tokens;
use qualifier::definitions::Definitions;
use std::fmt::Debug;
use type_checker::{env::Env, infer};

fn run_syntactic_frontend<'src, 'ident, 'ast>(
    text: &'src str,
    ident_bump: &'ident Bump,
    ast_bump: &'ast Bump,
) -> Result<ir::ast::Program<'ast, 'ident, &'ident str, ir::ast::Type<'ast, 'ident>>, String> {
    let (tokens, errors) = scan_tokens(text);
    if !errors.success() {
        return Err(format!(
            "while scanning {}:\n\tparsed tokens: {:?}\n\twith errors: {:?}",
            text, tokens, errors
        ));
    }
    let token_iter = tokens.into_iter();
    let ast_program =
        match parser::program::program(
            &mut token_iter.clone().peekable(),
            &General::new(ast_bump),
            &Interning::new(ident_bump),
        ) {
            Ok(Ok(ast)) => ast,
            Err(error) => {
                return Err(format!(
                "while parsing {}:\n\tparsed tokens: {:?}\n\tbut met irrefutable parse error: {:?}",
                text, token_iter.collect::<Vec<_>>(), error
            ));
            }
            Ok(Err(error)) => {
                return Err(format!(
                "while parsing {}:\n\tparsed tokens: {:?}\n\tbut met reffutable parse error: {:?}",
                text, token_iter.collect::<Vec<_>>(), error
            ));
            }
        };
    Ok(ast_program)
}

#[allow(dead_code)]
fn run_semantic_frontend<'src, 'ident, 'qual>(
    ast: ir::ast::Program<'_, 'ident, &'ident str, ir::ast::Type<'_, 'ident>>,
    ident: &'ident Bump,
    typed_tree: &'qual Bump,
) -> Result<ir::typed::Program<'qual, 'ident>, String> {
    let qualified_tree = Bump::new();
    let mut defs = Definitions::default();
    let qualified_program = match qualifier::qualifier::program(
        ast,
        &mut defs,
        &Interning::new(ident),
        &General::new(&qualified_tree),
    ) {
        Ok(prog) => prog,
        Err(error) => {
            return Err(format!(
                "while qualifying {:?}\n\twas met with qualification error {:?}",
                ast, error
            ));
        }
    };

    let mut env = Env::default();
    let typed_program = match infer::program(
        qualified_program,
        &mut env,
        &Interning::new(ident),
        &General::new(typed_tree),
    ) {
        Ok(prog) => prog,
        Err(error) => {
            return Err(format!(
                "while type checking {:?}\n\tqualified to {:?}\n\tbut was met with type error {:?}",
                ast, qualified_program, error
            ));
        }
    };

    Ok(typed_program)
}

#[derive(Copy, Clone)]
pub enum TestType {
    AllPass,
    AllFail,
}

fn run_test_over_data<D: Clone + Debug, T: Debug, E: Debug>(
    mut test: impl FnMut(D) -> Result<T, E>,
    test_name: &str,
    test_type: TestType,
    data: impl AsRef<[D]>,
) {
    let results: Vec<_> = data
        .as_ref()
        .iter()
        .map(|test_case| (test_case, test(test_case.clone())))
        .collect();

    let successes: Vec<_> = results
        .iter()
        .filter_map(|(test_case, test_result)| match test_result {
            Ok(result) => Some((test_case, result)),
            Err(_) => None,
        })
        .collect();

    let failures: Vec<_> = results
        .iter()
        .filter_map(|(test_case, test_result)| match test_result {
            Ok(_) => None,
            Err(error) => Some((test_case, error)),
        })
        .collect();

    match test_type {
        TestType::AllPass => {
            if !failures.is_empty() {
                let mut msg = format!("{} tests failed on level {}:\n", failures.len(), test_name);

                for (test_case, error) in failures {
                    msg += &format!("\n\n\ndata:\n{:?}\n\nerror:\n{:?}", test_case, error);
                }

                panic!("{}", msg)
            }
        }
        TestType::AllFail => {
            if !successes.is_empty() {
                let mut msg = format!(
                    "{} tests passed when they shouldn't have on level {}:\n",
                    successes.is_empty(),
                    test_name
                );

                for (test_case, result) in successes {
                    msg += &format!("\n\n\ndata:\n{:?}\n\nerror:\n{:?}", test_case, result);
                }

                panic!("{}", msg)
            }
        }
    }
}

const TRIVIAL_PROGRAMS: [&str; 10] = [
    "func f() = 0",
    "func f[t](a: t): t = a",
    "func f[t](a: t): t = {let x = a; x}",
    "func call[a, b](f: func(a): b, x: a): b = f(x)",
    "func ufcs[a](f: func(int, int): int): int = 3.f(5)",
    "func f() = {x: 5,}",
    "func wrap[a](x: a): {x: a,} = {x: x,}",
    "func add(x: int, y: int): int = 5",
    "func if[a](predicate: bool, branch_if: a, branch_else: a): a = branch_if",
    "func unbox[a]({inner: boxed,}: {inner: a,}): a = boxed",
];

#[test]
fn trivial_functions_syntax() {
    run_test_over_data(
        |text| {
            let bump = Bump::new();
            run_syntactic_frontend(text, &bump, &bump).map(|_| ())
        },
        "trivial syntax",
        TestType::AllPass,
        TRIVIAL_PROGRAMS,
    );
}

#[test]
fn trivial_functions_semantics() {
    run_test_over_data(
        |text| {
            let bump = Bump::new();
            let ast = run_syntactic_frontend(text, &bump, &bump).unwrap();
            run_semantic_frontend(ast, &bump, &bump).map(|_| ())
        },
        "trivial semantics",
        TestType::AllPass,
        TRIVIAL_PROGRAMS,
    )
}

#[test]
fn illegal_functions_semantics() {
    let bump = Bump::new();
    run_test_over_data(
        |test| {
            let ast = run_syntactic_frontend(test, &bump, &bump).unwrap();
            run_semantic_frontend(ast, &bump, &bump)
        },
        "illegal semantics",
        TestType::AllFail,
        [
            "func f() = x",
            "func f(x: A) = x",
            "func f[a](x: a): x = x",
            "func f[a](x: a): a = a",
            "func f() = {x: x,}",
        ],
    )
}
