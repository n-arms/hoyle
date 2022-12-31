use arena_alloc::{Interning, Specialized};
use ir::qualified::IdentifierSource;
use ir::typed::{Identifier, Type, UntypedIdentifier};
use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, Default)]
pub struct Bindings<'expr, 'ident> {
    variables: HashMap<UntypedIdentifier<'ident>, Identifier<'expr, 'ident>>,
    identifier_seed: usize,
}

impl<'expr, 'ident> Bindings<'expr, 'ident> {
    pub fn rebind_variable(
        &mut self,
        variable: impl Into<UntypedIdentifier<'ident>>,
        binding: Identifier<'expr, 'ident>,
    ) {
        self.variables.insert(variable.into(), binding);
    }

    pub fn lookup_variable(
        &self,
        variable: impl Into<UntypedIdentifier<'ident>>,
    ) -> Identifier<'expr, 'ident> {
        *self
            .variables
            .get(&variable.into())
            .expect("the frontend should have caught undefined variables")
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn generate_identifier(
        &mut self,
        r#type: Type<'expr, 'ident>,
        interner: &Interning<'ident, Specialized>,
    ) -> Identifier<'expr, 'ident> {
        let seed = self.identifier_seed;
        self.identifier_seed += 1;

        let name_length = (seed as f64).log10().ceil().trunc() as usize + 3;

        // buffer overflow attacks galore! don't use the `unchecked` family of methods on this
        let mut buffer = [0u8; 16];

        write!((&mut buffer) as &mut [u8], "var{seed}").unwrap();
        let buffer_str = std::str::from_utf8(&buffer[..name_length]).unwrap();
        let interned_str = interner.get_or_intern(buffer_str);

        Identifier {
            name: interned_str,
            source: IdentifierSource::Local,
            r#type,
        }
    }
}
