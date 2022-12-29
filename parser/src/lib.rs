#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::option_if_let_else,
    clippy::redundant_else
)]

pub mod expr;
pub mod pattern;
pub mod program;
pub mod types;
pub mod util;
