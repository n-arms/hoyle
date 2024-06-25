/*
use arena_alloc::General;
use ir::qualified::{self, Type};
use ir::typed::{Block, Branch, Expr, Field, Identifier, Pattern, PatternField, Statement};
use std::collections::HashMap;

#[derive(Default)]
pub struct Substitution<'expr>(HashMap<qualified::Identifier, Type<'expr>>);

impl<'expr> Substitution<'expr> {
    #[must_use]
    pub fn unit(from: qualified::Identifier, to: Type<'expr>) -> Self {
        let mut sub = Substitution::default();
        sub.substitute(from, to);
        sub
    }

    pub fn substitute(&mut self, from: qualified::Identifier, to: Type<'expr>) {
        self.0.insert(from, to);
    }

    pub fn union(&mut self, other: &Substitution<'expr>) {
        self.0.extend(&other.0);
    }

    #[must_use]
    pub fn lookup(&self, from: &qualified::Identifier) -> Option<Type<'expr>> {
        self.0.get(from).copied()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'expr> From<HashMap<qualified::Identifier, Type<'expr>>> for Substitution<'expr> {
    fn from(map: HashMap<qualified::Identifier, Type<'expr>>) -> Self {
        Self(map)
    }
}

impl<'expr> Substitute<'expr> for Identifier {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        Identifier {
            identifier: self.identifier,
            r#type: self.r#type.apply(sub, alloc),
        }
    }
}

impl<'expr> Substitute<'expr> for Type<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        match self {
            qualified::Type::Named { name, .. } => sub.lookup(name).unwrap_or(*self),
            qualified::Type::Arrow {
                arguments,
                return_type,
                span,
            } => Type::Arrow {
                arguments: alloc
                    .alloc_slice_fill_iter(arguments.iter().map(|arg| arg.apply(sub, alloc))),
                return_type: alloc.alloc(return_type.apply(sub, alloc)),
                span: *span,
            },
        }
    }
}

impl<'expr> Substitute<'expr> for Expr<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        match self {
            ir::source::Expr::Variable(identifier, span) => {
                Expr::Variable(identifier.apply(sub, alloc), *span)
            }
            ir::source::Expr::Literal(literal, span) => Expr::Literal(*literal, *span),
            ir::source::Expr::Call {
                function,
                arguments,
                span,
            } => Expr::Call {
                function: alloc.alloc(function.apply(sub, alloc)),
                arguments: alloc
                    .alloc_slice_fill_iter(arguments.iter().map(|arg| arg.apply(sub, alloc))),
                span: *span,
            },
            ir::source::Expr::Operation {
                operator,
                arguments,
                span,
            } => Expr::Operation {
                operator: *operator,
                arguments: alloc
                    .alloc_slice_fill_iter(arguments.iter().map(|arg| arg.apply(sub, alloc))),
                span: *span,
            },
            ir::source::Expr::StructLiteral { name, fields, span } => Expr::StructLiteral {
                name: name.apply(sub, alloc),
                fields: alloc
                    .alloc_slice_fill_iter(fields.iter().map(|field| field.apply(sub, alloc))),
                span: *span,
            },
            ir::source::Expr::Block(block) => Expr::Block(block.apply(sub, alloc)),
            ir::source::Expr::Annotated {
                expr,
                annotation,
                span,
            } => Expr::Annotated {
                expr: alloc.alloc(expr.apply(sub, alloc)),
                annotation: annotation.apply(sub, alloc),
                span: *span,
            },
            ir::source::Expr::Case {
                predicate,
                branches,
                span,
            } => Expr::Case {
                predicate: alloc.alloc(predicate.apply(sub, alloc)),
                branches: alloc
                    .alloc_slice_fill_iter(branches.iter().map(|branch| branch.apply(sub, alloc))),
                span: *span,
            },
        }
    }
}

impl<'expr> Substitute<'expr> for Field<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        Field {
            name: self.name,
            value: self.value.apply(sub, alloc),
            span: self.span,
        }
    }
}

impl<'expr> Substitute<'expr> for Branch<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        Branch {
            pattern: self.pattern.apply(sub, alloc),
            body: self.body.apply(sub, alloc),
            span: self.span,
        }
    }
}

impl<'expr> Substitute<'expr> for PatternField<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        Self {
            name: self.name.apply(sub, alloc),
            pattern: self.pattern.apply(sub, alloc),
            span: self.span,
        }
    }
}

impl<'expr> Substitute<'expr> for Pattern<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        match self {
            ir::source::Pattern::Variable(identifier, span) => {
                Pattern::Variable(identifier.apply(sub, alloc), *span)
            }
            ir::source::Pattern::Struct { name, fields, span } => {
                let sub_name = name.apply(sub, alloc);
                let sub_fields =
                    alloc.alloc_slice_fill_iter(fields.iter().map(|f| f.apply(sub, alloc)));
                Pattern::Struct {
                    name: sub_name,
                    fields: sub_fields,
                    span: *span,
                }
            }
        }
    }
}

impl<'expr> Substitute<'expr> for Block<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        Block {
            statements: alloc
                .alloc_slice_fill_iter(self.statements.iter().map(|stmt| stmt.apply(sub, alloc))),
            result: self
                .result
                .map(|result| alloc.alloc(result.apply(sub, alloc)) as &_),
            span: self.span,
        }
    }
}

impl<'expr> Substitute<'expr> for Statement<'expr> {
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self {
        match self {
            ir::source::Statement::Let {
                left_side,
                right_side,
                span,
            } => Statement::Let {
                left_side: left_side.apply(sub, alloc),
                right_side: right_side.apply(sub, alloc),
                span: *span,
            },
            ir::source::Statement::Raw(expr) => Statement::Raw(expr.apply(sub, alloc)),
        }
    }
}

pub trait Substitute<'expr> {
    #[must_use]
    fn apply(&self, sub: &Substitution<'expr>, alloc: &General<'expr>) -> Self;
}
*/
