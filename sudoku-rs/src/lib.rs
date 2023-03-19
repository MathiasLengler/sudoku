// rustc lints
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
// clippy
#![warn(clippy::pedantic)]
// too pedantic
#![allow(clippy::redundant_else)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::redundant_closure_for_method_calls)]
// TODO: fix separately
#![allow(clippy::trivially_copy_pass_by_ref)]
// TODO: fix separately
#![allow(clippy::must_use_candidate)]
// warn a subset of lints in clippy::restriction
#![warn(clippy::undocumented_unsafe_blocks)]

pub use crate::sudoku::*;

pub mod base;
pub mod cell;
pub mod error;
pub mod generator;
pub mod grid;
pub mod position;
pub mod samples;
pub mod solver;
mod sudoku;
