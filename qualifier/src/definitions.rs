use crate::error::{Error, Result};
use im::{hashmap, HashMap};
use ir::qualified::{self, Identifier, IdentifierSource, StructDefinition, TypeName};

#[derive(Clone)]
pub struct Definitions<'expr, 'ident> {
    variables: HashMap<&'ident str, Identifier<'expr, 'ident>>,
    types: HashMap<&'ident str, TypeName<'ident>>,
    structs: HashMap<&'ident str, StructDefinition<'expr, 'ident>>,
}

impl<'expr, 'ident> Default for Definitions<'expr, 'ident> {
    fn default() -> Self {
        Self {
            variables: HashMap::default(),
            types: hashmap![
                "int" => TypeName { source: IdentifierSource::Global(qualified::Path::Builtin), name: "int" },
                "bool" => TypeName { source: IdentifierSource::Global(qualified::Path::Builtin), name: "bool" }
            ],
            structs: HashMap::default(),
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

    pub fn with_struct(&mut self, name: &'ident str, qualified: StructDefinition<'expr, 'ident>) {
        self.structs.insert(name, qualified);
        self.types.insert(
            name,
            TypeName {
                source: IdentifierSource::Local,
                name,
            },
        );
    }

    pub fn lookup_variable(
        &self,
        variable: &'ident str,
    ) -> Result<'expr, 'ident, Identifier<'expr, 'ident>> {
        self.variables
            .get(variable)
            .copied()
            .ok_or(Error::UndefinedVariable(variable))
    }

    pub fn lookup_type(
        &self,
        r#type: &'ident str,
    ) -> Result<'expr, 'ident, qualified::TypeName<'ident>> {
        self.types
            .get(r#type)
            .copied()
            .ok_or(Error::UndefinedType(r#type))
    }

    pub fn lookup_struct(
        &self,
        r#struct: &'ident str,
    ) -> Result<'expr, 'ident, StructDefinition<'expr, 'ident>> {
        self.structs
            .get(r#struct)
            .copied()
            .ok_or(Error::UndefinedStruct(r#struct))
    }
}
