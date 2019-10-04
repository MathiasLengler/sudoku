use std::convert::TryInto;

use failure::bail;

use crate::cell::SudokuCell;
use crate::error::Result;
use crate::grid::Grid;

fn from_givens<Cell: SudokuCell>(input: &str) -> Result<Grid<Cell>> {
    input
        .chars()
        .map(|c| match c {
            '.' | '-' | '0' => Ok(0),
            c => match c.to_digit(10) {
                Some(digit) => Ok(digit.try_into()?),
                None => bail!("Unexpected character in givens: {}", c),
            },
        })
        .collect::<Result<Vec<_>>>()?
        .try_into()
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
}
