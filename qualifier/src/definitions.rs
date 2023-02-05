use crate::error::{Error, Result};
use ir::qualified::{
    FieldDefinition, Identifier, IdentifierSource, LocalTagSource, Path, Primitives,
    StructDefinition, Tag,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Global<'expr, 'ident> {
    types: HashMap<&'ident str, Identifier<'ident>>,
    structs: HashMap<&'ident str, StructDefinition<'expr, 'ident>>,
    import_paths: HashMap<Tag, IdentifierSource>,
}

#[derive(Clone, Debug)]
pub struct Local<'expr, 'ident, 'names> {
    definitions: Rc<RefCell<Global<'expr, 'ident>>>,
    variables: HashMap<&'ident str, Identifier<'ident>>,
    tags: LocalTagSource<'names>,
}
impl<'expr, 'ident, 'names> Local<'expr, 'ident, 'names> {
    #[must_use]
    pub fn new(tags: LocalTagSource<'names>, primitives: Primitives<'ident>) -> Self {
        Self {
            definitions: Rc::new(RefCell::new(Global::new(primitives))),
            variables: HashMap::default(),
            tags,
        }
    }

    pub fn define_local_variable(&mut self, variable: &'ident str) -> Identifier<'ident> {
        let mut defs = self.definitions.borrow_mut();
        let id = self.tags.fresh_identifier(variable);
        self.variables.insert(variable, id);
        defs.import_paths.insert(id.tag, IdentifierSource::Local);
        id
    }

    pub fn define_local_type(&mut self, r#type: &'ident str) -> Identifier<'ident> {
        let tag = self.tags.fresh_tag();
        self.definitions
            .borrow_mut()
            .define_type(r#type, IdentifierSource::Local, tag)
    }

    pub fn define_local_struct(
        &mut self,
        name: &'ident str,
        fields: &'expr [FieldDefinition<'expr, 'ident>],
    ) -> StructDefinition<'expr, 'ident> {
        let tag = self.tags.fresh_tag();
        self.definitions
            .borrow_mut()
            .define_struct(name, fields, IdentifierSource::Local, tag)
    }

    pub fn define_local_field(&mut self, name: &'ident str) -> Identifier<'ident> {
        let tag = self.tags.fresh_tag();
        self.definitions
            .borrow_mut()
            .define_field(name, IdentifierSource::Local, tag)
    }

    pub fn lookup_type<'old>(
        &self,
        r#type: &'ident str,
    ) -> Result<'old, 'expr, 'ident, Identifier<'ident>> {
        let res = self.definitions.borrow().lookup_type(r#type);
        res
    }

    pub fn lookup_struct<'old>(
        &self,
        r#struct: &'ident str,
    ) -> Result<'old, 'expr, 'ident, StructDefinition<'expr, 'ident>> {
        self.definitions.borrow().lookup_struct(r#struct)
    }

    pub fn lookup_variable<'old>(
        &self,
        variable: &'ident str,
    ) -> Result<'old, 'expr, 'ident, Identifier<'ident>> {
        self.variables
            .get(variable)
            .copied()
            .ok_or(Error::UndefinedVariable(variable))
    }
}

impl<'expr, 'ident> Global<'expr, 'ident> {
    #[must_use]
    pub fn new(primitives: Primitives<'ident>) -> Self {
        let mut defs = Self {
            types: HashMap::default(),
            structs: HashMap::default(),
            import_paths: HashMap::default(),
        };

        defs.define_primitive_type("int", primitives.int.tag);
        defs.define_primitive_type("bool", primitives.bool.tag);
        defs
    }

    pub fn define_type(
        &mut self,
        r#type: &'ident str,
        source: IdentifierSource,
        tag: Tag,
    ) -> Identifier<'ident> {
        let id = Identifier::new(tag, r#type);
        self.types.insert(r#type, id);
        self.import_paths.insert(id.tag, source);
        id
    }

    fn define_primitive_type(&mut self, r#type: &'ident str, tag: Tag) -> Identifier<'ident> {
        let id = Identifier::new(tag, r#type);
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
        tag: Tag,
    ) -> StructDefinition<'expr, 'ident> {
        let id = Identifier::new(tag, name);
        let def = StructDefinition { name: id, fields };
        self.structs.insert(name, def);
        self.types.insert(name, id);
        self.import_paths.insert(id.tag, source);
        def
    }

    pub fn define_field(
        &mut self,
        field: &'ident str,
        source: IdentifierSource,
        tag: Tag,
    ) -> Identifier<'ident> {
        let id = Identifier::new(tag, field);
        self.import_paths.insert(id.tag, source);
        id
    }

    pub fn lookup_type<'old>(
        &self,
        r#type: &'ident str,
    ) -> Result<'old, 'expr, 'ident, Identifier<'ident>> {
        self.types
            .get(r#type)
            .copied()
            .ok_or(Error::UndefinedType(r#type))
    }

    pub fn lookup_struct<'old>(
        &self,
        r#struct: &'ident str,
    ) -> Result<'old, 'expr, 'ident, StructDefinition<'expr, 'ident>> {
        self.structs
            .get(r#struct)
            .copied()
            .ok_or(Error::UndefinedStruct(r#struct))
    }
}
