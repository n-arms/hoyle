#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::option_if_let_else,
    clippy::redundant_else
)]

#[macro_use]
pub mod util;
pub mod program;

use chumsky::primitive::end;
use chumsky::{error::Simple, Parser};
use tree::parsed::*;
use tree::token::Token;

pub fn parse<'src>(tokens: &[Token<'src>]) -> Result<Program, Vec<Simple<Token<'src>>>> {
    program::program().then_ignore(end()).parse(tokens)
}
