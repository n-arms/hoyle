use crate::error::Result;
use crate::extract::struct_type;
use crate::unify;
use im::HashMap;

use ir::qualified;
use ir::typed::{FieldDefinition, Identifier, Type};

#[derive(Clone)]
pub struct Primitives<'expr, 'ident> {
    pub int: Type<'expr, 'ident>,
    pub bool: Type<'expr, 'ident>,
}

#[derive(Clone)]
pub struct Env<'expr, 'ident> {
    variables: HashMap<qualified::Identifier<'ident>, Identifier<'expr, 'ident>>,
    structs: HashMap<qualified::Identifier<'ident>, &'expr [FieldDefinition<'expr, 'ident>]>,
    pub primitives: Primitives<'expr, 'ident>,
}

impl<'expr, 'ident> Env<'expr, 'ident> {
    #[must_use]
    pub fn new(primitives: Primitives<'expr, 'ident>) -> Self {
        Self {
            primitives,
            variables: HashMap::default(),
            structs: HashMap::default(),
        }
    }
    pub fn bind_variable(
        &mut self,
        variable: qualified::Identifier<'ident>,
        r#type: Type<'expr, 'ident>,
    ) {
        let typed = Identifier {
            identifier: variable,
            r#type,
        };
        self.variables.insert(variable, typed);
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
        &self,
        variable: qualified::Identifier<'ident>,
    ) -> Identifier<'expr, 'ident> {
        *self
            .variables
            .get(&variable)
            .expect("the qualifier pass should have caught undefined variables")
    }

    pub fn check_variable(
        &self,
        identifier: qualified::Identifier<'ident>,
        target: Type<'expr, 'ident>,
    ) -> Result<'expr, 'ident, ()> {
        let typed_identifier = self.lookup_variable(identifier);

        unify::types(typed_identifier.r#type, target)
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
