use crate::error::{Error, Result};
use im::HashMap;
use ir::qualified::{self, Identifier};

#[derive(Clone, Default)]
pub struct Definitions<'expr, 'ident> {
    variables: HashMap<&'ident str, Identifier<'expr, 'ident>>,
    types: HashMap<&'ident str, qualified::Type<'expr, 'ident>>,
}

impl<'expr, 'ident> Definitions<'expr, 'ident> {
    pub fn with_variables<I>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (&'ident str, Identifier<'expr, 'ident>)>,
    {
        self.variables.extend(variables);
        self
    }

    pub fn with_types<I>(mut self, types: I) -> Self
    where
        I: IntoIterator<Item = (&'ident str, qualified::Type<'expr, 'ident>)>,
    {
        self.types.extend(types);
        self
    }

    pub fn lookup_variable(
        &self,
        variable: &'ident str,
    ) -> Result<'ident, Identifier<'expr, 'ident>> {
        self.variables
            .get(variable)
            .copied()
            .ok_or(Error::UndefinedVariable(variable))
    }

    pub fn lookup_type(
        &self,
        r#type: &'ident str,
    ) -> Result<'ident, qualified::Type<'expr, 'ident>> {
        self.types
            .get(r#type)
            .copied()
            .ok_or(Error::UndefinedType(r#type))
    }
}
