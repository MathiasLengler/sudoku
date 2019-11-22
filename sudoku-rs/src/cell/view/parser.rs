use std::convert::TryInto;

use crate::cell::view::CellView;
use crate::error::Result;

pub(crate) fn parse_cells(input: &str) -> Result<Vec<CellView>> {
    let input = input.trim();

    Ok(if input.contains('\n') {
        from_candidates(input).unwrap_or_else(|_| from_givens_grid(input))
    } else {
        from_givens_line(input)?
    })
}

fn from_givens_line(input: &str) -> Result<Vec<CellView>> {
    input
        .chars()
        .map(TryInto::<CellView>::try_into)
        .collect::<Result<Vec<CellView>>>()
}

fn from_givens_grid(input: &str) -> Vec<CellView> {
    input
        .chars()
        .map(TryInto::<CellView>::try_into)
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
}

fn from_candidates(input: &str) -> Result<Vec<CellView>> {
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
        .collect::<Result<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_givens_line_base_3() -> Result<()> {
        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/givens_line.txt"
        ));

        let cells = from_givens_line(input)?;

        let expected_cells = vec![
            6, 0, 0, 0, 0, 2, 3, 0, 0, 1, 2, 5, 6, 0, 0, 0, 0, 0, 0, 0, 4, 7, 0, 0, 0, 2, 0, 7, 3,
            0, 0, 0, 0, 8, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 6, 0, 0, 0, 0, 1, 5, 0, 5, 0, 0,
            0, 8, 1, 0, 0, 0, 0, 0, 0, 0, 3, 4, 7, 2, 0, 0, 7, 2, 0, 0, 0, 0, 8,
        ]
        .into_iter()
        .map(crate::cell::view::v)
        .collect::<Vec<_>>();

        assert_eq!(cells, expected_cells);

        Ok(())
    }

    #[test]
    fn test_givens_grid_base_3() -> Result<()> {
        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/givens_grid.txt"
        ));

        let cells = from_givens_grid(input);

        let expected_cells = vec![
            0, 8, 0, 5, 0, 3, 0, 7, 0, 0, 2, 7, 0, 0, 0, 3, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            5, 0, 9, 0, 6, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 0, 4, 0, 6, 0, 9, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 3, 2, 0, 0, 0, 4, 5, 0, 0, 5, 0, 9, 0, 7, 0, 2, 0,
        ]
        .into_iter()
        .map(crate::cell::view::v)
        .collect::<Vec<_>>();

        assert_eq!(cells, expected_cells);

        Ok(())
    }

    #[test]
    fn test_candidates() -> Result<()> {
        use crate::cell::view::{c, v};

        let input = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/res/candidates.txt"
        ));

        let cells = from_candidates(input)?;

        let expected_cells = vec![
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
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        assert_eq!(cells, expected_cells);

        Ok(())
    }
}
