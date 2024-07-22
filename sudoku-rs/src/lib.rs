// #![feature(iter_next_chunk)]
// #![feature(trusted_len)]
#![feature(generic_const_exprs)]
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
#![allow(clippy::too_many_lines)]
// TODO: fix separately
#![allow(clippy::must_use_candidate)]
// warn a subset of lints in clippy::restriction
#![warn(clippy::assertions_on_result_states)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::deref_by_slicing)]
#![warn(clippy::empty_enum_variants_with_brackets)]
#![warn(clippy::fn_to_numeric_cast_any)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::infinite_loop)]
#![warn(clippy::lossy_float_literal)]
#![warn(clippy::mem_forget)]
#![warn(clippy::mixed_read_write_in_expression)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::mutex_atomic)]
#![warn(clippy::needless_raw_strings)]
#![warn(clippy::pub_without_shorthand)]
#![warn(clippy::rc_buffer)]
#![warn(clippy::rc_mutex)]
#![warn(clippy::rest_pat_in_fully_bound_structs)]
#![warn(clippy::separated_literal_suffix)]
#![warn(clippy::string_add)]
#![warn(clippy::string_lit_chars_any)]
#![warn(clippy::string_slice)]
#![warn(clippy::string_to_string)]
#![warn(clippy::suspicious_xor_used_as_pow)]
#![warn(clippy::tests_outside_test_module)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::unneeded_field_pattern)]

pub use crate::sudoku::*;

pub mod base;
pub mod cell;
pub mod error;
pub mod generator;
pub mod grid;
pub mod position;
pub mod rng;
pub mod samples;
pub mod solver;
mod sudoku;
pub(crate) mod unsafe_utils;
pub mod world;

#[cfg(test)]
mod test_util;
