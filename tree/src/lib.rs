#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]

mod generic;
pub mod parsed;
pub mod sized;
pub mod token;
pub mod type_passing;
pub mod typed;

use smartstring::{LazyCompact, SmartString};

pub type String = SmartString<LazyCompact>;
