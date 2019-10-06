use std::convert::TryInto;

use crate::cell::view::CellView;
use crate::cell::SudokuCell;
use crate::error::Result;
use crate::grid::Grid;

pub(crate) fn from_givens<Cell: SudokuCell>(input: &str) -> Result<Grid<Cell>> {
    input
        .chars()
        .map(TryInto::<CellView>::try_into)
        .collect::<Result<Vec<_>>>()?
        .try_into()
}

pub(crate) fn from_candidates<Cell: SudokuCell>(input: &str) -> Result<Grid<Cell>> {
    let cell_views = input
        .lines()
        // Filter horizontal separator lines
        .filter(|line| line.contains(|c: char| c.is_digit(36)))
        // Filter vertical separators
        .flat_map(|line| line.split(['-', '|', ':', '+', '\'', '\n', '*'].as_ref()))
        .filter(|s| *s != "")
        // Split and trim groups of numbers
        .flat_map(|s| s.split_whitespace())
        .map(TryInto::<CellView>::try_into)
        .collect::<Result<Vec<_>>>()?;

    cell_views.try_into()
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    use super::*;

    #[test]
    fn test_givens_base_3() -> Result<()> {
        let input =
            "6....23..1256.......47...2.73....84...........46....15.5...81.......3472..72....8";

        let grid = from_givens::<Cell>(input)?;

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
    fn test_candidates() -> Result<()> {
        use crate::cell::view::{c, v};

        let input = ".--------------.----------------.------------.
| 6   7    89  | 189  19   2    | 3   5   4  |
| 1   2    5   | .    3    4    | 9   8   7  |
| 3   89   4   | 7    58   59   | 6   2   1  |
:--------------+----------------+------------:
| 7   3    29  | 19   25   1569 | 8   4   69 |
| 5   1    289 | 89   0    679  | 27  69  3  |
| 89  4    6   | 3    28   79   | 27  1   5  |
:--------------+----------------+------------:
| 2   5    3   | 4    7    8    | X   69  69 |
| 89  689  1   | 5    69   3    | 4   x   2  |
| 4   69   7   | 2    169  169  | 5   3   8  |
'--------------'----------------'------------'";

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
