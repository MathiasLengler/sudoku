use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::CellState;
use crate::error::Result;
use crate::grid::format::GridFormat;
use crate::grid::Grid;
use itertools::Itertools;
use tabled::builder::Builder;
use tabled::settings::{object::Segment, Alignment, Modify, Style};

/// A grid of cells.
/// Candidates are visualized as concatenated numbers in a single line.
/// The grid borders are represented by [UTF-8 box drawing characters](https://en.wikipedia.org/wiki/Box_Drawing).
///
/// # Example
///
/// TODO: update
/// ```text
/// .--------------.----------------.------------.
/// | 6   7    89  | 189  19   2    | 3   5   4  |
/// | 1   2    5   | .    3    4    | 9   8   7  |
/// | 3   89   4   | 7    58   59   | 6   2   1  |
/// :--------------+----------------+------------:
/// | 7   3    29  | 19   25   1569 | 8   4   69 |
/// | 5   1    289 | 89   0    679  | 27  69  3  |
/// | 89  4    6   | 3    28   79   | 27  1   5  |
/// :--------------+----------------+------------:
/// | 2   5    3   | 4    7    8    | 0   69  69 |
/// | 89  689  1   | 5    69   3    | 4   .   2  |
/// | 4   69   7   | 2    169  169  | 5   3   8  |
/// '--------------'----------------'------------'
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CandidatesGridCompact;

impl GridFormat for CandidatesGridCompact {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        let all_block_cells = grid
            .all_block_cells()
            .map(|block| block.collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let grid_builder: Builder = all_block_cells
            .chunks(usize::from(Base::BASE))
            .map(|row_of_blocks| {
                row_of_blocks.iter().map(|block| {
                    let block_builder: Builder = block
                        .chunks(usize::from(Base::BASE))
                        .map(|block_row| {
                            block_row.iter().map(|cell| match cell.state() {
                                CellState::Value(value) | CellState::FixedValue(value) => {
                                    value.to_string()
                                }
                                CellState::Candidates(candidates) => {
                                    if candidates.is_empty() {
                                        "0".to_string()
                                    } else {
                                        candidates
                                            .iter()
                                            .map(|candidate| candidate.to_string())
                                            .join("")
                                    }
                                }
                            })
                        })
                        .collect();
                    block_builder
                        .build()
                        .with(Modify::new(Segment::all()).with(Alignment::center()))
                        .with(Style::empty())
                        .to_string()
                })
            })
            .collect();

        let mut table = grid_builder.build();

        table
            .with(Modify::new(Segment::all()).with(Alignment::center()))
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
        static GRID_BORDER_CHARS: &[char] = &['-', '|', ':', '+', '\'', '\n', '*'];

        input
            .lines()
            // Filter horizontal separator lines
            .filter(|line| line.contains(|c: char| c.is_digit(36)))
            // Filter vertical separators
            .flat_map(|line| line.split(GRID_BORDER_CHARS))
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
        "/tests/res/grid_formats/candidates_grid_compact.txt"
    ));

    #[test]
    fn test_render() {
        todo!()
    }

    #[test]
    fn test_parse() -> Result<()> {
        use crate::cell::dynamic::{c, v};

        let cells = CandidatesGridCompact.parse(INPUT_CANDIDATES)?;

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
