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

use arena_alloc::{General, Interning, Specialized};
use ir::source::Program;
use ir::token::Token;
use std::iter::Peekable;
use util::Irrecoverable;

pub fn parse<'src, 'expr>(
    text: &mut Peekable<impl Iterator<Item = Token<'src>> + Clone>,
    alloc: &General<'expr>,
) -> Result<Program<'expr>, util::Irrecoverable> {
    let program = program::program(text, alloc)?.map_err(Irrecoverable::WhileParsingProgram)?;

    Ok(program)
}
