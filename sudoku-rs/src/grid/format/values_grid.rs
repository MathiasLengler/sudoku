use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::Result;
use crate::grid::Grid;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormat;
use crate::grid::format::GridFormatCapabilities;
use crate::grid::format::GridFormatDetectAndParseCapability;
use crate::grid::format::GridFormatPreservesCellCandidates;
use crate::grid::format::GridFormatPreservesCellValue;

/// A grid of cell values.
/// Candidates are displayed as `0`.
/// The grid borders are represented by [UTF-8 box drawing characters](https://en.wikipedia.org/wiki/Box_Drawing).
///
/// # Example
/// ```text
///  8 0 0 │ 0 0 0 │ 0 0 0
///  0 0 3 │ 6 0 0 │ 0 0 0
///  0 7 0 │ 0 9 0 │ 2 0 0
/// ───────┼───────┼───────
///  0 5 0 │ 0 0 7 │ 0 0 0
///  0 0 0 │ 0 4 5 │ 7 0 0
///  0 0 0 │ 1 0 0 │ 0 3 0
/// ───────┼───────┼───────
///  0 0 1 │ 0 0 0 │ 0 6 8
///  0 0 8 │ 5 0 0 │ 0 1 0
///  0 9 0 │ 0 0 0 │ 4 0 0
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ValuesGrid;

impl GridFormat for ValuesGrid {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueOnly,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::Empty,
            // Detected and parsed as `CandidatesGridCompact`
            detect_and_parse: GridFormatDetectAndParseCapability::DetectableViaOtherFormat,
        }
    }

    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        use itertools::Itertools;
        use tabled::builder::Builder;
        use tabled::settings::style::Style;

        let builder: Builder = grid
            .all_block_cells()
            .map(|block| {
                block
                    .chunks(usize::from(Base::BASE))
                    .into_iter()
                    .map(|block_row| block_row.map(|cell| cell.to_string()).join(" "))
                    .join("\n")
            })
            .chunks(usize::from(Base::BASE))
            .into_iter()
            .collect();

        builder
            .build()
            .with(
                Style::modern()
                    .remove_top()
                    .remove_left()
                    .remove_right()
                    .remove_bottom(),
            )
            .to_string()
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        input
            .chars()
            .map(TryInto::<DynamicCell>::try_into)
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
            .try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::consts::Base3, grid::format::test_util::assert_parsed_grid, samples};
    use indoc::indoc;

    #[test]
    fn test_render_givens_grid() {
        let grid = samples::base_3().into_iter().next().unwrap();

        assert_eq!(
            ValuesGrid.render(&grid),
            indoc! {"
                 8 0 0 │ 0 0 0 │ 0 0 0 
                 0 0 3 │ 6 0 0 │ 0 0 0 
                 0 7 0 │ 0 9 0 │ 2 0 0 
                ───────┼───────┼───────
                 0 5 0 │ 0 0 7 │ 0 0 0 
                 0 0 0 │ 0 4 5 │ 7 0 0 
                 0 0 0 │ 1 0 0 │ 0 3 0 
                ───────┼───────┼───────
                 0 0 1 │ 0 0 0 │ 0 6 8 
                 0 0 8 │ 5 0 0 │ 0 1 0 
                 0 9 0 │ 0 0 0 │ 4 0 0 "
            }
        );
    }

    #[test]
    fn test_from_givens_grid() {
        let cells = ValuesGrid
            .parse(indoc! {"
                *-----------*
                |.8.|5.3|.7.|
                |.27|...|38.|
                |...|...|...|
                |---+---+---|
                |..5|.9.|6..|
                |...|1.2|...|
                |..4|.6.|9..|
                |---+---+---|
                |...|...|...|
                |.32|...|45.|
                |.5.|9.7|.2.|
                *-----------*"
            })
            .unwrap();

        let expected_grid = Grid::<Base3>::try_from(vec![
            vec![0, 8, 0, 5, 0, 3, 0, 7, 0],
            vec![0, 2, 7, 0, 0, 0, 3, 8, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 5, 0, 9, 0, 6, 0, 0],
            vec![0, 0, 0, 1, 0, 2, 0, 0, 0],
            vec![0, 0, 4, 0, 6, 0, 9, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 3, 2, 0, 0, 0, 4, 5, 0],
            vec![0, 5, 0, 9, 0, 7, 0, 2, 0],
        ])
        .unwrap();

        assert_parsed_grid(&expected_grid, &cells).unwrap();
    }
}
