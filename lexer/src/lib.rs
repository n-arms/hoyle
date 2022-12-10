#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::redundant_else)]

pub mod lexer;
pub mod span_source;

pub use crate::lexer::scan_tokens;
