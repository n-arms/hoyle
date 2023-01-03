use arena_alloc::General;
use ir::qualified;
use ir::typed::{Block, Branch, Expr, Field, Identifier, Pattern, Statement, Type};
use std::collections::HashMap;

#[derive(Default)]
pub struct Substitution<'expr, 'ident>(HashMap<qualified::Identifier<'ident>, Type<'expr, 'ident>>);

impl<'expr, 'ident> Substitution<'expr, 'ident> {
    #[must_use]
    pub fn unit(from: qualified::Identifier<'ident>, to: Type<'expr, 'ident>) -> Self {
        let mut sub = Substitution::default();
        sub.substitute(from, to);
        sub
    }

    pub fn substitute(&mut self, from: qualified::Identifier<'ident>, to: Type<'expr, 'ident>) {
        self.0.insert(from, to);
    }

    pub fn union(&mut self, other: &Substitution<'expr, 'ident>) {
        self.0.extend(&other.0);
    }

    #[must_use]
    pub fn lookup(&self, from: &qualified::Identifier<'ident>) -> Option<Type<'expr, 'ident>> {
        self.0.get(from).copied()
    }
}

impl<'expr, 'ident> From<HashMap<qualified::Identifier<'ident>, Type<'expr, 'ident>>>
    for Substitution<'expr, 'ident>
{
    fn from(map: HashMap<qualified::Identifier<'ident>, Type<'expr, 'ident>>) -> Self {
        Self(map)
    }
}

impl<'expr, 'ident> Substitute<'expr, 'ident> for Identifier<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
        Identifier {
            identifier: self.identifier,
            r#type: self.r#type.apply(sub, alloc),
        }
    }
}

impl<'expr, 'ident> Substitute<'expr, 'ident> for Type<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
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

impl<'expr, 'ident> Substitute<'expr, 'ident> for Expr<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
        match self {
            ir::ast::Expr::Variable(identifier, span) => {
                Expr::Variable(identifier.apply(sub, alloc), *span)
            }
            ir::ast::Expr::Literal(literal, span) => Expr::Literal(*literal, *span),
            ir::ast::Expr::Call {
                function,
                arguments,
                span,
            } => Expr::Call {
                function: alloc.alloc(function.apply(sub, alloc)),
                arguments: alloc
                    .alloc_slice_fill_iter(arguments.iter().map(|arg| arg.apply(sub, alloc))),
                span: *span,
            },
            ir::ast::Expr::Operation {
                operator,
                arguments,
                span,
            } => Expr::Operation {
                operator: *operator,
                arguments: alloc
                    .alloc_slice_fill_iter(arguments.iter().map(|arg| arg.apply(sub, alloc))),
                span: *span,
            },
            ir::ast::Expr::StructLiteral { name, fields, span } => Expr::StructLiteral {
                name: name.apply(sub, alloc),
                fields: alloc
                    .alloc_slice_fill_iter(fields.iter().map(|field| field.apply(sub, alloc))),
                span: *span,
            },
            ir::ast::Expr::Block(block) => Expr::Block(block.apply(sub, alloc)),
            ir::ast::Expr::Annotated {
                expr,
                annotation,
                span,
            } => Expr::Annotated {
                expr: alloc.alloc(expr.apply(sub, alloc)),
                annotation: annotation.apply(sub, alloc),
                span: *span,
            },
            ir::ast::Expr::Case {
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

impl<'expr, 'ident> Substitute<'expr, 'ident> for Field<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
        Field {
            name: self.name,
            value: self.value.apply(sub, alloc),
            span: self.span,
        }
    }
}

impl<'expr, 'ident> Substitute<'expr, 'ident> for Branch<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
        Branch {
            pattern: self.pattern.apply(sub, alloc),
            body: self.body.apply(sub, alloc),
            span: self.span,
        }
    }
}

impl<'expr, 'ident> Substitute<'expr, 'ident> for Pattern<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
        match self {
            ir::ast::Pattern::Variable(identifier, span) => {
                Pattern::Variable(identifier.apply(sub, alloc), *span)
            }
            ir::ast::Pattern::Struct { name, fields, span } => todo!(),
        }
    }
}

impl<'expr, 'ident> Substitute<'expr, 'ident> for Block<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
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

impl<'expr, 'ident> Substitute<'expr, 'ident> for Statement<'expr, 'ident> {
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self {
        match self {
            ir::ast::Statement::Let {
                left_side,
                right_side,
                span,
            } => Statement::Let {
                left_side: left_side.apply(sub, alloc),
                right_side: right_side.apply(sub, alloc),
                span: *span,
            },
            ir::ast::Statement::Raw(expr, span) => Statement::Raw(expr.apply(sub, alloc), *span),
        }
    }
}

pub trait Substitute<'expr, 'ident> {
    #[must_use]
    fn apply(&self, sub: &Substitution<'expr, 'ident>, alloc: &General<'expr>) -> Self;
}
