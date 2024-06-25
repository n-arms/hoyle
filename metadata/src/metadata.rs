use arena_alloc::General;
use ir::metadata::{Function, Metadata, Struct};
use ir::qualified::LocalTagSource;
use ir::typed;

#[must_use]
pub fn program<'expr>(
    to_metadata: typed::Program<'expr>,
    tags: LocalTagSource,
    alloc: &General<'expr>,
) -> Metadata<'expr> {
    let mut metadata = Metadata::default();

    for def in to_metadata.definitions {
        definition(def, tags, alloc, &mut metadata);
    }

    metadata
}

pub fn definition<'expr>(
    to_metadata: &typed::Definition<'expr>,
    tags: LocalTagSource,
    alloc: &General<'expr>,
    metadata: &mut Metadata<'expr>,
) {
    match to_metadata {
        typed::Definition::Function(typed::FunctionDefinition { name, generics, .. }) => {
            let generic_args =
                alloc.alloc_slice_fill_iter(generics.iter().map(|generic| generic.name.clone()));

            metadata.functions.insert(
                name.clone(),
                Function {
                    generic_type: todo!(),
                    generic_args,
                },
            );
        }
        typed::Definition::Struct(typed::StructDefinition { name, .. }) => {
            let metadata_constructor = tags.fresh_tag();

            metadata.structs.insert(
                name.clone(),
                Struct {
                    metadata_constructor,
                },
            );
        }
    }
}
