use tree::type_passing::*;
use tree::typed;
use tree::String;

use crate::env::Env;

pub fn program(to_pass: &typed::Program) -> Program {
    let env = Env::default();
    let structs = to_pass
        .structs
        .iter()
        .map(|to_pass| strukt(&env, to_pass))
        .collect();
    let functions = to_pass
        .functions
        .iter()
        .map(|func| function(&env, func))
        .collect();
    Program { structs, functions }
}

fn strukt(env: &Env, to_pass: &typed::Struct) -> Struct {
    let fields = to_pass
        .fields
        .iter()
        .map(|field| typ(env, &field.typ))
        .collect();
    let tag = StructMeta {
        arguments: Vec::new(),
        fields,
    };
    Struct {
        name: to_pass.name.clone(),
        fields: to_pass.fields.clone(),
        tag,
    }
}

fn function(env: &Env, to_pass: &typed::Function) -> Function {
    let mut arguments = Vec::new();
    let result_arg = Argument {
        name: String::from("_result"),
        typ: to_pass.result.clone(),
    };
    arguments.push(result_arg);
    arguments.extend(to_pass.arguments.iter().cloned());
    for generic in &to_pass.generics {
        arguments.push(Argument {
            name: generic.name.clone(),
            typ: Type::typ(),
        });
    }
    Function {
        name: to_pass.name.clone(),
        generics: Vec::new(),
        arguments,
        result: to_pass.result.clone(),
        body: expr(env, &to_pass.body),
    }
}

fn expr(env: &Env, to_pass: &typed::Expr) -> Expr {
    match to_pass {
        typed::Expr::Variable { name, typ } => Expr::Variable {
            name: name.clone(),
            typ: typ.clone(),
        },
        typed::Expr::Literal { literal } => Expr::Literal {
            literal: literal.clone(),
        },
        typed::Expr::CallDirect {
            function,
            arguments,
            tag,
        } => {
            let mut passed_args = Vec::new();
            for arg in arguments {
                passed_args.push(expr(env, arg));
            }
            for arg in &tag.generics {
                passed_args.push(typ(env, arg));
            }
            Expr::CallDirect {
                function: function.clone(),
                arguments: passed_args,
                tag: Call {
                    result: tag.result.clone(),
                    signature: make_signature(arguments.len()),
                },
            }
        }
        typed::Expr::Block(to_pass) => Expr::Block(block(env, to_pass)),
        typed::Expr::Primitive {
            primitive,
            arguments,
        } => {
            let lowered_args = arguments.iter().map(|arg| expr(env, arg)).collect();
            Expr::Primitive {
                primitive: *primitive,
                arguments: lowered_args,
            }
        }
        typed::Expr::StructPack { name, fields, tag } => {
            let passed_fields = fields
                .iter()
                .map(|field| PackField {
                    name: field.name.clone(),
                    value: expr(env, &field.value),
                })
                .collect();
            Expr::StructPack {
                name: name.clone(),
                fields: passed_fields,
                tag: tag.clone(),
            }
        }
        typed::Expr::If {
            predicate,
            true_branch,
            false_branch,
            tag,
        } => {
            let passed_predicate = expr(env, &predicate);
            let passed_true = expr(env, &true_branch);
            let passed_false = expr(env, &false_branch);
            Expr::If {
                predicate: Box::new(passed_predicate),
                true_branch: Box::new(passed_true),
                false_branch: Box::new(passed_false),
                tag: *tag,
            }
        }
    }
}

fn block(env: &Env, to_pass: &typed::Block) -> Block {
    let mut passed_stmts = Vec::new();
    for stmt in &to_pass.stmts {
        let passed_stmt = match stmt {
            typed::Statement::Let { name, typ, value } => Statement::Let {
                name: name.clone(),
                typ: typ.clone(),
                value: expr(env, value),
            },
        };
        passed_stmts.push(passed_stmt);
    }
    Block {
        stmts: passed_stmts,
        result: Box::new(expr(env, &to_pass.result)),
    }
}

fn typ(env: &Env, to_pass: &Type) -> Expr {
    match to_pass {
        Type::Named { name, arguments } => {
            let passed_args = arguments.iter().map(|arg| typ(env, arg)).collect();
            Expr::CallDirect {
                function: name.clone(),
                arguments: passed_args,
                tag: Call {
                    result: Type::typ(),
                    signature: make_signature(arguments.len()),
                },
            }
        }
        Type::Generic { name } => Expr::Variable {
            name: name.clone(),
            typ: Type::typ(),
        },
        Type::Function { arguments, result } => {
            let mut passed_args = Vec::new();
            passed_args.extend(arguments.iter().map(|arg| typ(env, arg)));
            passed_args.push(typ(env, result));
            Expr::CallDirect {
                function: n_function(arguments.len()),
                arguments: passed_args,
                tag: Call {
                    result: Type::typ(),
                    signature: make_signature(arguments.len() + 1),
                },
            }
        }
    }
}

fn n_function(arity: usize) -> String {
    String::from(format!("{}function", arity))
}

fn make_signature(arity: usize) -> Vec<Convention> {
    let mut signature = vec![Convention::Out];
    signature.extend(vec![Convention::In; arity]);
    signature
}
