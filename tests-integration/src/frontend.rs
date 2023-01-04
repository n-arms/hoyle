use arena_alloc::*;
use bumpalo::Bump;
use ir::{
    ast::Field,
    qualified::{TagSource, Type},
};
use lexer::scan_tokens;
use qualifier::definitions::Local;
use std::fmt::{Debug, Display};
use type_checker::{
    check,
    env::{Env, Primitives},
    infer,
};

fn run_syntactic_frontend<'src, 'ident, 'ast>(
    text: &'src str,
    ident_bump: &'ident Bump,
    ast_bump: &'ast Bump,
) -> Result<ir::ast::Program<'ast, &'ident str, &'ident str>, String> {
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

fn extract_primitives<'old, 'new, 'ident>(defs: Local<'old, 'ident>) -> Primitives<'new, 'ident> {
    Primitives {
        int: Type::Named {
            name: defs.lookup_type("int").unwrap(),
            span: None,
        },
        bool: Type::Named {
            name: defs.lookup_type("bool").unwrap(),
            span: None,
        },
    }
}

#[allow(dead_code)]
fn run_semantic_frontend<'src, 'ident, 'qual>(
    ast: ir::ast::Program<'_, &'ident str, &'ident str>,
    ident: &'ident Bump,
    typed_tree: &'qual Bump,
) -> Result<ir::typed::Program<'qual, 'ident>, String> {
    let qualified_tree = Bump::new();
    let tags = TagSource::default();
    let mut defs = Local::new(1, tags.clone());
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

    let mut env = Env::new(tags, extract_primitives(defs));
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

    crate::check_qualified::program(typed_program)?;

    Ok(typed_program)
}

#[derive(Copy, Clone)]
pub enum TestType {
    AllPass,
    AllFail,
}

fn run_test_over_data<D: Clone + Debug, T: Debug, E: Display>(
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
                    msg += &format!("\n\n\ndata:\n{:?}\n\nerror:\n{}", test_case, error);
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

const TRIVIAL_PROGRAMS: &[&str] = &[
    "func f() = 0",
    "func f[t](a: t): t = a",
    "func f[t](a: t): t = {let x = a; x}",
    "func call[a, b](f: func(a): b, x: a): b = f(x)",
    "func ufcs[a](f: func(int, int): int): int = 3.f(5)",
    "
    struct x_wrapper {
        x: int
    }
    func f() = x_wrapper {x: 5}
    ",
    "func add(x: int, y: int): int = 5",
    "func if[a](predicate: bool, branch_if: a, branch_else: a): a = branch_if",
    "func pattern_id[a](x: a): a = case x of { val => val }",
    "
    struct wrapper {
        x: int
    }
    func unwrap_in_args(wrapper {x: y}: wrapper): int = y
    func unwrap_in_let(w: wrapper): int = {
        let wrapper {x: y} = w;
        y
    }
    func unwrap_in_case(w: wrapper): int = case w of { wrapper {x: y} => y }
    ",
    "
    func id[a](x: a): a = x
    func five(): int = id(5)
    ",
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
            "func f(x: a) = x",
            "func f[a](x: a): x = x",
            "func f[a](x: a): a = a",
            "func f() = x_wrapper {x: x}",
            "func f() = unknown {x: 5}",
        ],
    )
}
