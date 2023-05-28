use anyhow::bail;

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::Cell;
use crate::error::Result;
use crate::grid::format::GridFormat;
use crate::grid::Grid;

/// # Example
/// `800000000003600000070090200050007000000045700000100030001000068008500010090000400`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GivensLine;

impl GridFormat for GivensLine {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        grid.all_cells().map(ToString::to_string).collect()
    }

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>> {
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

        match input.chars().count() {
            BASE_2_CHAR_COUNT => parse_base::<Base2>(input),
            BASE_3_CHAR_COUNT => parse_base::<Base3>(input),
            BASE_4_CHAR_COUNT => parse_base::<Base4>(input),
            BASE_5_CHAR_COUNT => parse_base::<Base5>(input),
            unexpected_char_count => bail!("Unexpected char count: {unexpected_char_count}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    pub(crate) static INPUT_GIVENS_LINE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/res/grid_formats/givens_line.txt"
    ));

    #[test]
    fn test_render_givens_line() {
        let grid = samples::base_3().pop().unwrap();

        assert_eq!(
            GivensLine.render(&grid),
            "800000000003600000070090200050007000000045700000100030001000068008500010090000400"
        );
    }

    #[test]
    fn test_from_givens_line() -> Result<()> {
        let cells = GivensLine.parse(INPUT_GIVENS_LINE)?;

        let expected_cells = vec![
            6, 0, 0, 0, 0, 2, 3, 0, 0, 1, 2, 5, 6, 0, 0, 0, 0, 0, 0, 0, 4, 7, 0, 0, 0, 2, 0, 7, 3,
            0, 0, 0, 0, 8, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 6, 0, 0, 0, 0, 1, 5, 0, 5, 0, 0,
            0, 8, 1, 0, 0, 0, 0, 0, 0, 0, 3, 4, 7, 2, 0, 0, 7, 2, 0, 0, 0, 0, 8,
        ]
        .into_iter()
        .map(crate::cell::dynamic::v)
        .collect::<Vec<_>>();

        assert_eq!(cells, expected_cells);

        Ok(())
    }
}
