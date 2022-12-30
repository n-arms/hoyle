use crate::error::*;
use crate::unify::unify_types;
use im::HashMap;
use ir::qualified;
use ir::typed::*;

#[derive(Clone, Default)]
pub struct Env<'expr, 'ident> {
    variables: HashMap<UntypedIdentifier<'ident>, Identifier<'expr, 'ident>>,
    structs: HashMap<UntypedIdentifier<'ident>, &'expr [FieldDefinition<'expr, 'ident>]>,
}

impl<'expr, 'ident> Env<'expr, 'ident> {
    pub fn bind_variable(
        &mut self,
        variable: impl Into<UntypedIdentifier<'ident>>,
        r#type: Type<'expr, 'ident>,
    ) {
        let untyped = variable.into();
        let typed = Identifier {
            source: untyped.source,
            name: untyped.name,
            r#type,
        };
        self.variables.insert(untyped, typed);
    }

    pub fn bind_variables<ID>(
        &mut self,
        bindings: impl IntoIterator<Item = (ID, Type<'expr, 'ident>)>,
    ) where
        ID: Into<UntypedIdentifier<'ident>>,
    {
        for (variable, r#type) in bindings {
            self.bind_variable(variable, r#type);
        }
    }

    pub fn bind_struct(
        &mut self,
        name: impl Into<UntypedIdentifier<'ident>>,
        fields: &'expr [FieldDefinition<'expr, 'ident>],
    ) {
        self.structs.insert(name.into(), fields);
    }

    pub fn lookup_variable(
        &self,
        variable: impl Into<UntypedIdentifier<'ident>>,
    ) -> Identifier<'expr, 'ident> {
        *self
            .variables
            .get(&variable.into())
            .expect("the qualifier pass should have caught undefined variables")
    }

    pub fn check_variable(
        &self,
        identifier: impl Into<UntypedIdentifier<'ident>>,
        target: Type<'expr, 'ident>,
    ) -> Result<'expr, 'ident, ()> {
        let typed_identifier = self.lookup_variable(identifier);

        unify_types(typed_identifier.r#type, target)
    }

    pub fn lookup_struct(
        &self,
        name: impl Into<UntypedIdentifier<'ident>>,
    ) -> &'expr [FieldDefinition<'expr, 'ident>] {
        self.structs
            .get(&name.into())
            .expect("the qualifier pass should have caught undefined structs")
    }
}
