#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

pub mod check;
pub mod env;
pub mod error;
pub mod extract;
pub mod infer;
pub mod substitute;
pub mod unify;
