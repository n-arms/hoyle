#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::option_if_let_else,
    clippy::redundant_else
)]

#[macro_use]
pub mod util;
pub mod program;

use chumsky::{error::Simple, Parser};
use ir::token::Token;
use tree::parsed::*;

pub fn parse<'src>(tokens: &[Token<'src>]) -> Result<Program, Vec<Simple<Token<'src>>>> {
    program::program().parse(tokens)
}
