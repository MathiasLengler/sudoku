// FIXME: sudokuwiki's format has changed again:
//  https://www.sudokuwiki.org/Sudoku_String_Definitions
//  https://blueant1.github.io/puzzle-coding/documentation/puzzlecoding/encodingformats/
//  => Implement as a new format

use crate::base::consts::*;
use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::{Candidates, Cell};
use crate::error::Result;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormat;
use crate::grid::format::GridFormatCapabilities;
use crate::grid::format::GridFormatDetectAndParseCapability;
use crate::grid::format::GridFormatPreservesCellCandidates;
use crate::grid::format::GridFormatPreservesCellValue;
use crate::grid::Grid;
use anyhow::bail;
use std::fmt::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BinaryCandidatesLineV2;

impl GridFormat for BinaryCandidatesLineV2 {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueAndFixedState,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::All,
            detect_and_parse: GridFormatDetectAndParseCapability::Detectable,
        }
    }
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        todo!()
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: https://www.sudokuwiki.org/Test_Strings

    #[ignore = "New format with header, not yet implemented"]
    #[test]
    fn test_parse_example() {
        BinaryCandidatesLineV2.parse("S9B015y2e685w68050609040i022e0e0f0a2e085y050f0a5u090b042e2u2e0i06042c0810012q0f0dd0015w9i102e020a089e03050f9e0d5y042e05d0609i010f095y0e5y0f0a045y0206020166cy669id205").unwrap();
    }
}
