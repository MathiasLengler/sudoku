use std::convert::TryInto;

use failure::bail;

use crate::cell::SudokuCell;
use crate::error::Result;
use crate::grid::Grid;

fn is_empty_cell_char(c: char) -> bool {
    match c {
        '.' | '-' | '0' | 'x' | 'X' => true,
        _ => false,
    }
}

fn char_value_to_usize(c: char) -> Result<usize> {
    if is_empty_cell_char(c) {
        Ok(0)
    } else {
        match c.to_digit(36) {
            Some(digit) => Ok(digit.try_into()?),
            None => bail!("Unable to convert character into number: {}", c),
        }
    }
}

fn candidates_str_to_cell<Cell: SudokuCell>(candidates: &str, max: usize) -> Result<Cell> {
    match candidates.len() {
        0 => bail!("Unexpected empty string while parsing candidates"),
        1 => Ok(Cell::new_with_value(
            char_value_to_usize(candidates.chars().next().unwrap())?,
            max,
        )),
        _ => Ok(Cell::new_with_candidates(
            candidates
                .chars()
                .map(|candidate| {
                    let candidate = char_value_to_usize(candidate)?;
                    if candidate == 0 {
                        bail!("A candidate can't be 0")
                    } else {
                        Ok(candidate)
                    }
                })
                .collect::<Result<Vec<_>>>()?,
            max,
        )),
    }
}

fn from_givens<Cell: SudokuCell>(input: &str) -> Result<Grid<Cell>> {
    input
        .chars()
        .map(char_value_to_usize)
        .collect::<Result<Vec<_>>>()?
        .try_into()
}

fn from_candidates<Cell: SudokuCell>(input: &str) -> Result<Grid<Cell>> {
    let vec_candidates_str = input
        .lines()
        // Filter horizontal separator lines
        .filter(|line| line.contains(|c: char| c.is_digit(36)))
        .flat_map(|line| line.split(['-', '|', ':', '+', '\'', '\n', '*'].as_ref()))
        .filter(|s| *s != "")
        .flat_map(|s| s.split_whitespace())
        .collect::<Vec<_>>();

    let base = Grid::<Cell>::cell_count_to_base(vec_candidates_str.len())?;
    let max = Grid::<Cell>::base_to_max_value(base);

    println!("{:#?}", vec_candidates_str);
    println!("{}", vec_candidates_str.len());

    Ok(Grid::new_with_cells(
        base,
        vec_candidates_str
            .into_iter()
            .map(|candidates_str| candidates_str_to_cell(candidates_str, max))
            .collect::<Result<_>>()?,
    ))
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

        // TODO: assert (nested CellView vec)

        println!("{}", grid);

        Ok(())
    }
}
