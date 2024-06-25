use crate::error::{Error, Result};
use ir::qualified::{
    FieldDefinition, Identifier, IdentifierSource, LocalTagSource, Path, Primitives,
    StructDefinition, Tag,
};
use ir::source::{self, Span};
use smartstring::SmartString;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Global<'expr> {
    types: HashMap<source::Identifier, Identifier>,
    structs: HashMap<source::Identifier, StructDefinition<'expr>>,
    import_paths: HashMap<Tag, IdentifierSource>,
}

#[derive(Clone, Debug)]
pub struct Local<'expr, 'names> {
    definitions: Rc<RefCell<Global<'expr>>>,
    variables: HashMap<source::Identifier, Identifier>,
    tags: LocalTagSource<'names>,
}
impl<'expr, 'names> Local<'expr, 'names> {
    #[must_use]
    pub fn new(tags: LocalTagSource<'names>, primitives: Primitives) -> Self {
        Self {
            definitions: Rc::new(RefCell::new(Global::new(primitives))),
            variables: HashMap::default(),
            tags,
        }
    }

    pub fn define_local_variable(&mut self, variable: source::Identifier) -> Identifier {
        let mut defs = self.definitions.borrow_mut();
        let id = self.tags.fresh_identifier(variable.name.clone());
        defs.import_paths.insert(id.tag, IdentifierSource::Local);
        self.variables.insert(variable, id.clone());
        id
    }

    pub fn define_local_type(&mut self, r#type: source::Identifier) -> Identifier {
        let tag = self.tags.fresh_tag();
        self.definitions
            .borrow_mut()
            .define_type(r#type, IdentifierSource::Local, tag)
    }

    pub fn define_local_struct(
        &mut self,
        name: source::Identifier,
        span: Span,
        fields: &'expr [FieldDefinition<'expr>],
    ) -> StructDefinition<'expr> {
        let tag = self.tags.fresh_tag();
        self.definitions.borrow_mut().define_struct(
            name,
            fields,
            IdentifierSource::Local,
            span,
            tag,
        )
    }

    pub fn define_local_field(&mut self, name: source::Identifier) -> Identifier {
        let tag = self.tags.fresh_tag();
        self.definitions
            .borrow_mut()
            .define_field(name, IdentifierSource::Local, tag)
    }

    pub fn lookup_type<'old>(&self, r#type: source::Identifier) -> Result<'old, 'expr, Identifier> {
        let res = self.definitions.borrow().lookup_type(r#type);
        res
    }

    pub fn lookup_struct<'old>(
        &self,
        r#struct: source::Identifier,
    ) -> Result<'old, 'expr, StructDefinition<'expr>> {
        self.definitions.borrow().lookup_struct(r#struct)
    }

    pub fn lookup_variable<'old>(
        &self,
        variable: source::Identifier,
    ) -> Result<'old, 'expr, Identifier> {
        self.variables
            .get(&variable)
            .cloned()
            .ok_or(Error::UndefinedVariable(variable))
    }
}

impl<'expr> Global<'expr> {
    #[must_use]
    pub fn new(primitives: Primitives) -> Self {
        let mut defs = Self {
            types: HashMap::default(),
            structs: HashMap::default(),
            import_paths: HashMap::default(),
        };

        defs.define_primitive_type(
            source::Identifier {
                name: primitives.integer.name,
            },
            primitives.integer.tag,
        );
        defs.define_primitive_type(
            source::Identifier {
                name: primitives.boolean.name,
            },
            primitives.boolean.tag,
        );
        defs
    }

    pub fn define_type(
        &mut self,
        r#type: source::Identifier,
        source: IdentifierSource,
        tag: Tag,
    ) -> Identifier {
        let id = Identifier::new(tag, r#type.clone());
        self.import_paths.insert(id.tag, source);
        self.types.insert(r#type, id.clone());
        id
    }

    fn define_primitive_type(&mut self, r#type: source::Identifier, tag: Tag) -> Identifier {
        let id = Identifier::new(tag, r#type.clone());
        self.import_paths
            .insert(id.tag, IdentifierSource::Global(Path::Builtin));
        self.types.insert(r#type, id.clone());

        id
    }

    pub fn define_struct(
        &mut self,
        name: source::Identifier,
        fields: &'expr [FieldDefinition<'expr>],
        source: IdentifierSource,
        span: Span,
        tag: Tag,
    ) -> StructDefinition<'expr> {
        let id = Identifier::new(tag, name.clone());
        self.import_paths.insert(id.tag, source);
        let def = StructDefinition {
            name: id.clone(),
            fields,
            span,
        };
        self.structs.insert(name.clone(), def.clone());
        self.types.insert(name, id);
        def
    }

    pub fn define_field(
        &mut self,
        field: source::Identifier,
        source: IdentifierSource,
        tag: Tag,
    ) -> Identifier {
        let id = Identifier::new(tag, field);
        self.import_paths.insert(id.tag, source);
        id
    }

    pub fn lookup_type<'old>(&self, r#type: source::Identifier) -> Result<'old, 'expr, Identifier> {
        self.types
            .get(&r#type)
            .cloned()
            .ok_or(Error::UndefinedType(r#type))
    }

    pub fn lookup_struct<'old>(
        &self,
        r#struct: source::Identifier,
    ) -> Result<'old, 'expr, StructDefinition<'expr>> {
        self.structs
            .get(&r#struct)
            .cloned()
            .ok_or(Error::UndefinedStruct(r#struct))
    }
}
