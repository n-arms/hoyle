#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

pub mod check;
pub mod env;
pub mod infer;
pub mod specialize;
pub mod unify;
