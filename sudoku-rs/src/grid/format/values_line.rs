use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::Cell;
use crate::error::Result;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormat;
use crate::grid::format::GridFormatCapabilities;
use crate::grid::format::GridFormatDetectAndParseCapability;
use crate::grid::format::GridFormatPreservesCellCandidates;
use crate::grid::format::GridFormatPreservesCellValue;
use crate::grid::Grid;
use anyhow::bail;

/// All grid values concatenated into a single line.
/// Candidates are displayed as `0`.
///
/// # Example
/// `800000000003600000070090200050007000000045700000100030001000068008500010090000400`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ValuesLine;

impl GridFormat for ValuesLine {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueOnly,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::Empty,
            detect_and_parse: GridFormatDetectAndParseCapability::Detectable,
        }
    }

    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        grid.all_cells().map(ToString::to_string).collect()
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        fn parse_base<Base: SudokuBase>(input: &str) -> Result<Vec<DynamicCell>> {
            input
                .chars()
                .map(|c| {
                    let dynamic_cell = DynamicCell::try_from(c)?;
                    let cell = Cell::<Base>::try_from(dynamic_cell)?;
                    Ok(DynamicCell::from(cell))
                })
                .collect::<Result<Vec<DynamicCell>>>()
        }

        use crate::base::consts::*;

        const BASE_2_CHAR_COUNT: usize = Base2::CELL_COUNT as usize;
        const BASE_3_CHAR_COUNT: usize = Base3::CELL_COUNT as usize;
        const BASE_4_CHAR_COUNT: usize = Base4::CELL_COUNT as usize;
        const BASE_5_CHAR_COUNT: usize = Base5::CELL_COUNT as usize;

        let dynamic_cells = match input.chars().count() {
            BASE_2_CHAR_COUNT => parse_base::<Base2>(input),
            BASE_3_CHAR_COUNT => parse_base::<Base3>(input),
            BASE_4_CHAR_COUNT => parse_base::<Base4>(input),
            BASE_5_CHAR_COUNT => parse_base::<Base5>(input),
            unexpected_char_count => bail!("Unexpected char count: {unexpected_char_count}"),
        }?;

        dynamic_cells.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::consts::Base3, grid::format::test_util::assert_parsed_grid, samples};

    #[test]
    fn test_render_givens_line() {
        let grid = samples::base_3().into_iter().next().unwrap();

        assert_eq!(
            ValuesLine.render(&grid),
            "800000000003600000070090200050007000000045700000100030001000068008500010090000400"
        );
    }

    #[test]
    fn test_from_givens_line() {
        let cells = ValuesLine
            .parse(
                "6....23..1256.......47...2.73....84...........46....15.5...81.......3472..72....8",
            )
            .unwrap();

        let expected_grid = Grid::<Base3>::try_from(vec![
            6, 0, 0, 0, 0, 2, 3, 0, 0, 1, 2, 5, 6, 0, 0, 0, 0, 0, 0, 0, 4, 7, 0, 0, 0, 2, 0, 7, 3,
            0, 0, 0, 0, 8, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 6, 0, 0, 0, 0, 1, 5, 0, 5, 0, 0,
            0, 8, 1, 0, 0, 0, 0, 0, 0, 0, 3, 4, 7, 2, 0, 0, 7, 2, 0, 0, 0, 0, 8,
        ])
        .unwrap();
        assert_parsed_grid(&expected_grid, &cells).unwrap();
    }
}
