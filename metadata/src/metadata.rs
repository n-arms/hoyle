use arena_alloc::General;
use ir::metadata::{Function, Metadata, Struct};
use ir::qualified::{Identifier, LocalTagSource, Type};
use ir::typed;

pub fn program<'expr, 'ident>(
    to_metadata: typed::Program<'expr, 'ident>,
    tags: LocalTagSource,
    alloc: &General<'expr>,
) -> Metadata<'expr, 'ident> {
    let mut metadata = Metadata::default();

    for def in to_metadata.definitions {
        definition(*def, tags, alloc, &mut metadata);
    }

    metadata
}

pub fn definition<'expr, 'ident>(
    to_metadata: typed::Definition<'expr, 'ident>,
    tags: LocalTagSource,
    alloc: &General<'expr>,
    metadata: &mut Metadata<'expr, 'ident>,
) {
    match to_metadata {
        typed::Definition::Function { name, generics, .. } => {
            let generic_args =
                alloc.alloc_slice_fill_iter(generics.iter().map(|generic| generic.identifier));

            metadata.functions.insert(
                name.identifier,
                Function {
                    generic_type: name.r#type,
                    generic_args,
                },
            );
        }
        typed::Definition::Struct { name, .. } => {
            let metadata_constructor = tags.fresh_tag();

            metadata.structs.insert(
                name.identifier,
                Struct {
                    metadata_constructor,
                },
            );
        }
    }
}
