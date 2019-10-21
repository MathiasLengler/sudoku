#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![deny(unsafe_code)]

#[cfg(feature = "flame_it")]
extern crate flame;
#[cfg(feature = "flame_it")]
#[macro_use]
extern crate flamer;

//pub use sudoku::*;
pub use sudoku::dynamic::DynamicSudoku;
pub use sudoku::Sudoku;

pub mod base;
pub mod cell;
pub mod error;
pub mod generator;
pub mod grid;
mod history;
pub mod position;
pub mod samples;
pub mod solver;
mod sudoku;
pub mod transport;
