use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::Result;
use crate::grid::format::GridFormat;
use crate::grid::Grid;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AsciiCandidatesGrid;

impl GridFormat for AsciiCandidatesGrid {
    fn render<Base: SudokuBase>(self, _grid: &Grid<Base>) -> String {
        todo!()
    }

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>> {
        static SEPARATORS: &[char] = &['-', '|', ':', '+', '\'', '\n', '*'];

        input
            .lines()
            // Filter horizontal separator lines
            .filter(|line| line.contains(|c: char| c.is_digit(36)))
            // Filter vertical separators
            .flat_map(|line| line.split(SEPARATORS))
            .filter(|s| !s.is_empty())
            // Split and trim groups of numbers
            .flat_map(|s| s.split_whitespace())
            .map(TryInto::<DynamicCell>::try_into)
            .collect::<Result<Vec<_>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) static INPUT_CANDIDATES: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/res/grid_formats/ascii_candidates_grid.txt"
    ));

    #[test]
    fn test_from_candidates_grid() -> Result<()> {
        use crate::cell::dynamic::{c, v};

        let cells = AsciiCandidatesGrid.parse(INPUT_CANDIDATES)?;

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
