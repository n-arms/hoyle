#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::option_if_let_else
)]

pub mod env;
pub mod lower;
pub mod refcount;
