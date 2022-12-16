#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::redundant_else,
    clippy::option_if_let_else
)]

pub mod general;
pub mod interning;

pub use general::General;
pub use interning::{GeneralPurpose, Interning, Specialized};
