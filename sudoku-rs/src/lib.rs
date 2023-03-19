#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

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
