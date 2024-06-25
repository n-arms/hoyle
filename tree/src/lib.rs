#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]

mod generic;
pub mod parsed;
pub mod typed;

use smartstring::{LazyCompact, SmartString};

pub type String = SmartString<LazyCompact>;
