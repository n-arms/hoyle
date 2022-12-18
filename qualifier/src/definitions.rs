use crate::error::{Error, Result};
use im::{hashmap, HashMap};
use ir::qualified::{self, Identifier};

#[derive(Clone)]
pub struct Definitions<'expr, 'ident> {
    variables: HashMap<&'ident str, Identifier<'expr, 'ident>>,
    types: HashMap<&'ident str, qualified::TypeName<'ident>>,
}

impl<'expr, 'ident> Default for Definitions<'expr, 'ident> {
    fn default() -> Self {
        Self {
            variables: HashMap::default(),
            types: hashmap![
                "int" => qualified::TypeName { source: qualified::IdentifierSource::Global(qualified::Path::Builtin), name: "int" },
                "bool" => qualified::TypeName { source: qualified::IdentifierSource::Global(qualified::Path::Builtin), name: "bool" }
            ],
        }
    }
}

impl<'expr, 'ident> Definitions<'expr, 'ident> {
    pub fn with_variables<I>(&mut self, variables: I)
    where
        I: IntoIterator<Item = (&'ident str, Identifier<'expr, 'ident>)>,
    {
        self.variables.extend(variables);
    }

    pub fn with_types<I>(&mut self, types: I)
    where
        I: IntoIterator<Item = (&'ident str, qualified::TypeName<'ident>)>,
    {
        self.types.extend(types);
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

    pub fn lookup_type(&self, r#type: &'ident str) -> Result<'ident, qualified::TypeName<'ident>> {
        self.types
            .get(r#type)
            .copied()
            .ok_or(Error::UndefinedType(r#type))
    }
}
