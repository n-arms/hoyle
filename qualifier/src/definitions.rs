use crate::error::{Error, Result};
use ir::ast::Span;
use ir::qualified::{
    FieldDefinition, Identifier, IdentifierSource, Path, StructDefinition, Tag, Type,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct GlobalDefinitions<'expr, 'ident> {
    types: HashMap<&'ident str, Identifier<'ident>>,
    structs: HashMap<&'ident str, StructDefinition<'expr, 'ident>>,
    import_paths: HashMap<Tag, IdentifierSource>,
    tags: TagSource,
}

#[derive(Clone, Debug)]
pub struct Definitions<'expr, 'ident> {
    definitions: Rc<RefCell<GlobalDefinitions<'expr, 'ident>>>,
    variables: HashMap<&'ident str, Identifier<'ident>>,
    module: u32,
}

#[derive(Debug, Default)]
struct TagSource {
    unused_identifier: u32,
}

impl TagSource {
    pub fn fresh_tag(&mut self, module: u32) -> Tag {
        let tag = Tag {
            module,
            identifier: self.unused_identifier,
        };
        self.unused_identifier += 1;
        tag
    }

    pub fn fresh_identifier<'ident>(
        &mut self,
        name: &'ident str,
        module: u32,
    ) -> Identifier<'ident> {
        Identifier {
            tag: self.fresh_tag(module),
            name,
        }
    }
}

impl<'expr, 'ident> Definitions<'expr, 'ident> {
    pub fn new(module: u32, definitions: GlobalDefinitions<'expr, 'ident>) -> Self {
        Self {
            definitions: Rc::new(RefCell::new(definitions)),
            variables: HashMap::default(),
            module,
        }
    }

    pub fn define_local_variable(&mut self, variable: &'ident str) -> Identifier<'ident> {
        let mut defs = self.definitions.borrow_mut();
        let id = defs.tags.fresh_identifier(variable, self.module);
        self.variables.insert(variable, id);
        defs.import_paths.insert(id.tag, IdentifierSource::Local);
        id
    }

    pub fn define_local_type(&mut self, r#type: &'ident str) -> Identifier<'ident> {
        self.definitions
            .borrow_mut()
            .define_type(r#type, IdentifierSource::Local, self.module)
    }

    pub fn define_local_struct(
        &mut self,
        name: &'ident str,
        fields: &'expr [FieldDefinition<'expr, 'ident>],
    ) -> StructDefinition<'expr, 'ident> {
        self.definitions.borrow_mut().define_struct(
            name,
            fields,
            IdentifierSource::Local,
            self.module,
        )
    }

    pub fn lookup_type(&self, r#type: &'ident str) -> Result<'expr, 'ident, Identifier<'ident>> {
        let res = self.definitions.borrow().lookup_type(r#type);
        /*
        if let Err(e) = res {
            panic!("{:?}", e);
        }
        */
        res
    }

    pub fn lookup_struct(
        &self,
        r#struct: &'ident str,
    ) -> Result<'expr, 'ident, StructDefinition<'expr, 'ident>> {
        self.definitions.borrow().lookup_struct(r#struct)
    }

    pub fn lookup_variable(
        &self,
        variable: &'ident str,
    ) -> Result<'expr, 'ident, Identifier<'ident>> {
        self.variables
            .get(variable)
            .copied()
            .ok_or(Error::UndefinedVariable(variable))
    }
}

impl<'expr, 'ident> Default for GlobalDefinitions<'expr, 'ident> {
    fn default() -> Self {
        let mut defs = Self {
            types: HashMap::default(),
            structs: HashMap::default(),
            tags: TagSource::default(),
            import_paths: HashMap::default(),
        };

        defs.define_primitive_type("int");
        defs.define_primitive_type("bool");
        defs
    }
}

impl<'expr, 'ident> GlobalDefinitions<'expr, 'ident> {
    pub fn define_type(
        &mut self,
        r#type: &'ident str,
        source: IdentifierSource,
        module: u32,
    ) -> Identifier<'ident> {
        let id = self.tags.fresh_identifier(r#type, module);
        self.types.insert(r#type, id);
        self.import_paths.insert(id.tag, source);
        id
    }

    fn define_primitive_type(&mut self, r#type: &'ident str) -> Identifier<'ident> {
        let id = self.tags.fresh_identifier(r#type, 0);
        self.types.insert(r#type, id);
        self.import_paths
            .insert(id.tag, IdentifierSource::Global(Path::Builtin));
        id
    }

    pub fn define_struct(
        &mut self,
        name: &'ident str,
        fields: &'expr [FieldDefinition<'expr, 'ident>],
        source: IdentifierSource,
        module: u32,
    ) -> StructDefinition<'expr, 'ident> {
        let id = self.tags.fresh_identifier(name, module);
        let def = StructDefinition { name: id, fields };
        self.structs.insert(name, def);
        self.types.insert(name, id);
        self.import_paths.insert(id.tag, source);
        def
    }

    pub fn lookup_type(&self, r#type: &'ident str) -> Result<'expr, 'ident, Identifier<'ident>> {
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
