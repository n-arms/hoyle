use im::HashMap;
use ir::qualified::Type;
use ir::typed::*;

#[derive(Clone, Default)]
pub struct Env<'expr, 'ident> {
    typed_variables: HashMap<UntypedIdentifier<'ident>, Identifier<'expr, 'ident>>,
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
        self.typed_variables.insert(untyped, typed);
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

    pub fn lookup_variable(
        &mut self,
        variable: impl Into<UntypedIdentifier<'ident>>,
    ) -> Identifier<'expr, 'ident> {
        *self
            .typed_variables
            .get(&variable.into())
            .expect("the qualifier pass should have got undefined variables")
    }
}
