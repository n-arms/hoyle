use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn annotated(args: TokenStream, input: TokenStream) -> TokenStream {
    todo!()
}
