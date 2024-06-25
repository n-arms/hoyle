use ir::qualified;
use ir::qualified::Primitives;
use ir::typed;
use std::collections::HashMap;

pub struct Scheme<'expr> {
    pub for_all: Vec<qualified::Identifier>,
    pub r#type: typed::Type<'expr>,
}

pub struct Env<'expr> {
    variables: HashMap<qualified::Identifier, Scheme<'expr>>,
    structs: HashMap<qualified::Identifier, typed::StructDefinition<'expr>>,
    pub primitives: Primitives,
}

fn unwrap_none<A>(value: Option<A>) {
    assert!(matches!(value, None));
}

impl<'expr> Env<'expr> {
    #[must_use]
    pub fn new(primitives: Primitives) -> Self {
        Self {
            variables: HashMap::default(),
            structs: HashMap::default(),
            primitives,
        }
    }

    pub fn define_variable(&mut self, identifier: qualified::Identifier, scheme: Scheme<'expr>) {
        unwrap_none(self.variables.insert(identifier, scheme))
    }

    pub fn define_struct(&mut self, definition: typed::StructDefinition<'expr>) {
        unwrap_none(self.structs.insert(definition.name.clone(), definition))
    }

    pub fn lookup_variable(&mut self, identifier: &qualified::Identifier) -> &Scheme<'expr> {
        self.variables.get(identifier).unwrap()
    }

    pub fn lookup_struct(
        &mut self,
        name: &qualified::Identifier,
    ) -> &typed::StructDefinition<'expr> {
        self.structs.get(name).unwrap()
    }
}

/*
use crate::error::{Error, Result};
use crate::extract::struct_type;
use crate::substitute::{Substitute, Substitution};
use arena_alloc::General;
use ir::qualified::{self, LocalTagSource, Primitives, Type};
use ir::typed::{FieldDefinition, Identifier};
use std::collections::HashMap;

pub struct QualifiedIdentifier<'expr, > {
    forall: Vec<qualified::Identifier<>>,
    identifier: Identifier<'expr, >,
}

pub struct Env<'expr, , 'names> {
    variables: HashMap<qualified::Identifier<>, QualifiedIdentifier<'expr, >>,
    structs: HashMap<qualified::Identifier<>, &'expr [FieldDefinition<'expr, >]>,
    pub primitives: Primitives<>,
    pub tags: LocalTagSource<'names>,
}

impl<'expr, , 'names> Env<'expr, , 'names> {
    #[must_use]
    pub fn new(tags: LocalTagSource<'names>, primitives: Primitives<>) -> Self {
        Self {
            primitives,
            variables: HashMap::default(),
            structs: HashMap::default(),
            tags,
        }
    }

    pub fn bind_unqualified_variable(
        &mut self,
        variable: qualified::Identifier<>,
        r#type: Type<'expr, >,
    ) {
        self.bind_qualified_variable(variable, r#type, Vec::default());
    }

    pub fn bind_qualified_variable(
        &mut self,
        variable: qualified::Identifier<>,
        r#type: Type<'expr, >,
        forall: Vec<qualified::Identifier<>>,
    ) {
        let typed = Identifier {
            identifier: variable,
            r#type,
        };
        let qualified = QualifiedIdentifier {
            forall,
            identifier: typed,
        };
        self.variables.insert(variable, qualified);
    }

    pub fn bind_struct(
        &mut self,
        name: qualified::Identifier<>,
        fields: &'expr [FieldDefinition<'expr, >],
    ) -> Identifier<'expr, > {
        self.structs.insert(name, fields);

        Identifier {
            r#type: struct_type(name),
            identifier: name,
        }
    }

    #[must_use]
    pub fn lookup_variable(
        &mut self,
        variable: qualified::Identifier<>,
        alloc: &General<'expr>,
    ) -> Identifier<'expr, > {
        let qualified = self
            .variables
            .get(&variable)
            .expect("the qualifier pass should have caught undefined variables");

        let sub: Substitution = qualified
            .forall
            .iter()
            .map(|type_var| {
                let new_type_var = self.tags.fresh_identifier("unification");
                (
                    *type_var,
                    Type::Named {
                        name: new_type_var,
                        span: None,
                    },
                )
            })
            .collect::<HashMap<_, _>>()
            .into();
        qualified.identifier.apply(&sub, alloc)
    }

    #[must_use]
    pub fn lookup_struct(
        &self,
        name: qualified::Identifier<>,
    ) -> &'expr [FieldDefinition<'expr, >] {
        self.structs
            .get(&name)
            .expect("the qualifier pass should have caught undefined structs")
    }
}

pub fn substitute_types<'expr, >(
    general: Type<'expr, >,
    specific: Type<'expr, >,
) -> Result<'expr, , Substitution<'expr, >> {
    match (general, specific) {
        (
            Type::Named { name, .. },
            Type::Named {
                name: target_name, ..
            },
        ) if target_name == name => Ok(Substitution::default()),
        (Type::Named { name, .. }, specific) => Ok(Substitution::unit(name, specific)),
        (
            Type::Arrow {
                arguments,
                return_type,
                ..
            },
            Type::Arrow {
                arguments: specific_arguments,
                return_type: specific_return_type,
                ..
            },
        ) => {
            let mut sub = substitute_types(*return_type, *specific_return_type)?;

            for (arg, specific_arg) in arguments.iter().zip(specific_arguments) {
                sub.union(&substitute_types(*arg, *specific_arg)?);
            }

            Ok(sub)
        }
        (found @ Type::Arrow { .. }, expected) => Err(Error::TypeMismatch { expected, found }),
    }
}

#[must_use]
pub fn is_unification(identifier: qualified::Identifier) -> bool {
    identifier.name == "unification"
}
*/
