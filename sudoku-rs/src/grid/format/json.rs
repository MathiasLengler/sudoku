use crate::{
    base::SudokuBase,
    cell::dynamic::DynamicCell,
    error::Result,
    grid::{
        Grid,
        dynamic::DynamicGrid,
        format::{
            GridFormat, GridFormatCapabilities, GridFormatDetectAndParseCapability,
            GridFormatPreservesCellCandidates, GridFormatPreservesCellValue,
        },
    },
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Json;

impl GridFormat for Json {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueAndFixedState,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::All,
            detect_and_parse: GridFormatDetectAndParseCapability::Detectable,
        }
    }
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        let dynamic_grid = DynamicGrid::<DynamicCell>::from(grid);
        serde_json::to_string(&dynamic_grid).expect("serialization to JSON should not fail")
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        Ok(serde_json::from_str(input)?)
    }
}
