#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::option_if_let_else
)]

pub mod definitions;
pub mod error;
pub mod qualifier;

pub use qualifier::program as qualify;
