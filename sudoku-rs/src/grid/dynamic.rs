use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::{DynamicBase, SudokuBase};
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;

// FIXME: should this struct contain an instance of DynamicBase?
//  two different use-cases:
//  - use as serde-compatible parameters/return with ts_rs
//  - "Controller" containing runtime state for base
//
//  implementation strategies:
//  - define struct/enum using serde-compatible types and ts_rs.
//    more user-friendly data represenation (no bit fiddling etc.)
//    - sometimes contains base state:
//      - DynamicGeneratorSettings (to specifiy to *new* base)
//      - TransportSudoku (to inform the UI about the *current* base)
//    - more often, does not:
//      - DynamicCell
//      - DynamicValue
//      - DynamicPosition
//  - enum variant for each base containing base-generic implementation,
//    define trait with actions using base-agnostic types. Examples:
//    - DynamicSudoku / DynamicSudokuActions => Sudoku<Base>
//    - DynamicCellWorld / DynamicCellWorldActions => CellWorld<Base>
// Factors:
// - is the type used primarily as a DTO / parameter type?
// - are its methods used/wrapped by sudoku-wasm for the UI?
//   - more like a "Controller"
// - used for writing test cases to reduce validation overhead?
// - does the type validate that only correct instances are constructed?
//   - if not, does the type indicate that it is in a (in)valid state?
//   - can the type be even validated in isolation? (e.g. without a provided base?)
// Additional confusion: we use the name prefix "Dynamic" for both:
// - enums listing all trait implementations of unit structs
//   Here the prefix is used to disambiguate the enum from the trait.
//   - DynamicStrategy (serde string enum)
//   - DynamicGridFormat (serde string enum)
//   - DynamicBase (could become serde number enum 2|3|4|5)
// - base "agnostic" types
//   Here the prefix is used to disambiguate the type from its base-generic equivalent.
//   - DynamicCell
//   - DynamicValue
//   - DynamicPosition
// The distiction between DTOs and "Controller"-like types is not clear.
//
// Can/should we abstract some of these relationships with custom conversion traits?
#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicGrid<T = DynamicCell> {
    // TODO: see above
    // base: DynamicBase,
    cells: Vec<T>,
}

impl<Base: SudokuBase> TryFrom<DynamicGrid> for Grid<Base> {
    type Error = Error;

    fn try_from(dynamic_grid: DynamicGrid) -> Result<Self> {
        dynamic_grid.cells.try_into()
    }
}
