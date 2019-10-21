use std::convert::TryInto;

use crate::base::SudokuBase;
use crate::cell::view::CellView;
use crate::error::Result;
use crate::grid::Grid;

// TODO: split into:
//  &str -> Result<Vec<CellView>>
//  Vec<CellView> -> Result<Grid<Base>> (TryFrom impl in Grid)

pub(crate) fn from_givens_line<Base: SudokuBase>(input: &str) -> Result<Grid<Base>> {
    input
        .chars()
        .map(TryInto::<CellView>::try_into)
        .collect::<Result<Vec<_>>>()?
        .try_into()
}

pub(crate) fn from_givens_grid<Base: SudokuBase>(input: &str) -> Result<Grid<Base>> {
    input
        .chars()
        .map(TryInto::<CellView>::try_into)
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
        .try_into()
}

pub(crate) fn from_candidates<Base: SudokuBase>(input: &str) -> Result<Grid<Base>> {
    input
        .lines()
        // Filter horizontal separator lines
        .filter(|line| line.contains(|c: char| c.is_digit(36)))
        // Filter vertical separators
        .flat_map(|line| line.split(['-', '|', ':', '+', '\'', '\n', '*'].as_ref()))
        .filter(|s| *s != "")
        // Split and trim groups of numbers
        .flat_map(|s| s.split_whitespace())
        .map(TryInto::<CellView>::try_into)
        .collect::<Result<Vec<_>>>()?
        .try_into()
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    use super::*;

    #[test]
    fn test_givens_line_base_3() -> Result<()> {
        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/givens_line.txt"
        ));

        let grid = from_givens_line::<Cell>(input)?;

        let expected_grid = vec![
            6, 0, 0, 0, 0, 2, 3, 0, 0, 1, 2, 5, 6, 0, 0, 0, 0, 0, 0, 0, 4, 7, 0, 0, 0, 2, 0, 7, 3,
            0, 0, 0, 0, 8, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 6, 0, 0, 0, 0, 1, 5, 0, 5, 0, 0,
            0, 8, 1, 0, 0, 0, 0, 0, 0, 0, 3, 4, 7, 2, 0, 0, 7, 2, 0, 0, 0, 0, 8,
        ]
        .try_into()?;

        assert_eq!(grid, expected_grid);

        Ok(())
    }

    #[test]
    fn test_givens_grid_base_3() -> Result<()> {
        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/givens_grid.txt"
        ));

        let grid = from_givens_grid::<Cell>(input)?;

        let expected_grid = vec![
            0, 8, 0, 5, 0, 3, 0, 7, 0, 0, 2, 7, 0, 0, 0, 3, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            5, 0, 9, 0, 6, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 0, 4, 0, 6, 0, 9, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 3, 2, 0, 0, 0, 4, 5, 0, 0, 5, 0, 9, 0, 7, 0, 2, 0,
        ]
        .try_into()?;

        assert_eq!(grid, expected_grid);

        Ok(())
    }

    #[test]
    fn test_candidates() -> Result<()> {
        use crate::cell::view::{c, v};

        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/candidates.txt"
        ));

        let grid = from_candidates::<Cell>(input)?;

        let expected_grid = vec![
            vec![
                v(6),
                v(7),
                c(vec![8, 9]),
                c(vec![1, 8, 9]),
                c(vec![1, 9]),
                v(2),
                v(3),
                v(5),
                v(4),
            ],
            vec![v(1), v(2), v(5), v(0), v(3), v(4), v(9), v(8), v(7)],
            vec![
                v(3),
                c(vec![8, 9]),
                v(4),
                v(7),
                c(vec![5, 8]),
                c(vec![5, 9]),
                v(6),
                v(2),
                v(1),
            ],
            vec![
                v(7),
                v(3),
                c(vec![2, 9]),
                c(vec![1, 9]),
                c(vec![2, 5]),
                c(vec![1, 5, 6, 9]),
                v(8),
                v(4),
                c(vec![6, 9]),
            ],
            vec![
                v(5),
                v(1),
                c(vec![2, 8, 9]),
                c(vec![8, 9]),
                v(0),
                c(vec![6, 7, 9]),
                c(vec![2, 7]),
                c(vec![6, 9]),
                v(3),
            ],
            vec![
                c(vec![8, 9]),
                v(4),
                v(6),
                v(3),
                c(vec![2, 8]),
                c(vec![7, 9]),
                c(vec![2, 7]),
                v(1),
                v(5),
            ],
            vec![
                v(2),
                v(5),
                v(3),
                v(4),
                v(7),
                v(8),
                v(0),
                c(vec![6, 9]),
                c(vec![6, 9]),
            ],
            vec![
                c(vec![8, 9]),
                c(vec![6, 8, 9]),
                v(1),
                v(5),
                c(vec![6, 9]),
                v(3),
                v(4),
                v(0),
                v(2),
            ],
            vec![
                v(4),
                c(vec![6, 9]),
                v(7),
                v(2),
                c(vec![1, 6, 9]),
                c(vec![1, 6, 9]),
                v(5),
                v(3),
                v(8),
            ],
        ]
        .try_into()?;

        assert_eq!(grid, expected_grid);

        Ok(())
    }
}
