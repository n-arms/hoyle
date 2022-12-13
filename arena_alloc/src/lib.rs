#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::redundant_else)]

pub mod general;
pub mod interning;

pub use general::General;
pub use interning::{GeneralPurpose, Interning, Specialized};
