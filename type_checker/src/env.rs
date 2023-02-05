use crate::error::{Error, Result};
use crate::extract::struct_type;
use crate::substitute::{Substitute, Substitution};
use arena_alloc::General;
use ir::qualified::{self, LocalTagSource, Primitives, Type};
use ir::typed::{FieldDefinition, Identifier};
use std::collections::HashMap;

pub struct QualifiedIdentifier<'expr, 'ident> {
    forall: Vec<qualified::Identifier<'ident>>,
    identifier: Identifier<'expr, 'ident>,
}

pub struct Env<'expr, 'ident, 'names> {
    variables: HashMap<qualified::Identifier<'ident>, QualifiedIdentifier<'expr, 'ident>>,
    structs: HashMap<qualified::Identifier<'ident>, &'expr [FieldDefinition<'expr, 'ident>]>,
    pub primitives: Primitives<'ident>,
    pub tags: LocalTagSource<'names>,
}

impl<'expr, 'ident, 'names> Env<'expr, 'ident, 'names> {
    #[must_use]
    pub fn new(tags: LocalTagSource<'names>, primitives: Primitives<'ident>) -> Self {
        Self {
            primitives,
            variables: HashMap::default(),
            structs: HashMap::default(),
            tags,
        }
    }

    pub fn bind_unqualified_variable(
        &mut self,
        variable: qualified::Identifier<'ident>,
        r#type: Type<'expr, 'ident>,
    ) {
        self.bind_qualified_variable(variable, r#type, Vec::default());
    }

    pub fn bind_qualified_variable(
        &mut self,
        variable: qualified::Identifier<'ident>,
        r#type: Type<'expr, 'ident>,
        forall: Vec<qualified::Identifier<'ident>>,
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
        name: qualified::Identifier<'ident>,
        fields: &'expr [FieldDefinition<'expr, 'ident>],
    ) -> Identifier<'expr, 'ident> {
        self.structs.insert(name, fields);

        Identifier {
            r#type: struct_type(name),
            identifier: name,
        }
    }

    #[must_use]
    pub fn lookup_variable(
        &mut self,
        variable: qualified::Identifier<'ident>,
        alloc: &General<'expr>,
    ) -> Identifier<'expr, 'ident> {
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
        name: qualified::Identifier<'ident>,
    ) -> &'expr [FieldDefinition<'expr, 'ident>] {
        self.structs
            .get(&name)
            .expect("the qualifier pass should have caught undefined structs")
    }
}

pub fn substitute_types<'expr, 'ident>(
    general: Type<'expr, 'ident>,
    specific: Type<'expr, 'ident>,
) -> Result<'expr, 'ident, Substitution<'expr, 'ident>> {
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

pub fn is_unification(identifier: qualified::Identifier) -> bool {
    identifier.name == "unification"
}
