use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::Result;
use crate::grid::format::GridFormat;
use crate::grid::Grid;

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
pub struct GivensGrid;

impl GridFormat for GivensGrid {
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

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>> {
        Ok(input
            .chars()
            .map(TryInto::<DynamicCell>::try_into)
            .filter_map(Result::ok)
            .collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    pub(crate) static INPUT_GIVENS_GRID: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/res/grid_formats/givens_grid.txt"
    ));

    #[test]
    fn test_render_givens_grid() {
        let grid = samples::base_3().into_iter().next().unwrap();

        assert_eq!(
            GivensGrid.render(&grid),
            " 8 0 0 │ 0 0 0 │ 0 0 0 
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
        );
    }

    #[test]
    fn test_from_givens_grid() {
        let cells = GivensGrid.parse(INPUT_GIVENS_GRID).unwrap();

        let expected_cells = vec![
            0, 8, 0, 5, 0, 3, 0, 7, 0, 0, 2, 7, 0, 0, 0, 3, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            5, 0, 9, 0, 6, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 0, 4, 0, 6, 0, 9, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 3, 2, 0, 0, 0, 4, 5, 0, 0, 5, 0, 9, 0, 7, 0, 2, 0,
        ]
        .into_iter()
        .map(crate::cell::dynamic::v)
        .collect::<Vec<_>>();

        assert_eq!(cells, expected_cells);
    }
}
