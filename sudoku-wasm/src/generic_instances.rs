//! Signature types for instances of generic types.
//!
//! - A derived `Tsify` on a generic type prints the bare generic name in wasm-bindgen signatures (tsify#76).
//! - Each newtype wraps the Rust alias, which declares the instance itself (`tsify::declare`).

use serde::{Deserialize, Serialize};
use sudoku::cell::dynamic::DynamicCell;
use sudoku::solver::strategic::strategies::selection::StrategySet;
use sudoku::world::{WorldCellPosition, WorldGridDim, WorldGridPosition};
use tsify::Tsify;

// Special case: `Vec` has no `Tsify` impl, and the README's `Vec<Ts<T>>` converts per element (up to 7% slower on this hot path).
#[tsify::declare]
pub type DynamicCells = Vec<DynamicCell>;

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct WasmDynamicCells(pub DynamicCells);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct WasmStrategySet(pub StrategySet);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct WasmWorldCellPosition(pub WorldCellPosition);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct WasmWorldGridPosition(pub WorldGridPosition);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[serde(transparent)]
pub struct WasmWorldGridDim(pub WorldGridDim);
