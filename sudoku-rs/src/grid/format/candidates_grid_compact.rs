use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::CellState;
use crate::error::Result;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormat;
use crate::grid::format::GridFormatCapabilities;
use crate::grid::format::GridFormatDetectAndParseCapability;
use crate::grid::format::GridFormatPreservesCellCandidates;
use crate::grid::format::GridFormatPreservesCellValue;
use crate::grid::Grid;
use itertools::Itertools;
use tabled::builder::Builder;
use tabled::settings::Style;

/// A grid of cells.
/// Candidates are visualized as concatenated numbers in a single line.
/// The grid borders are represented by [UTF-8 box drawing characters](https://en.wikipedia.org/wiki/Box_Drawing).
///
/// # Example
///
/// ```text
///   8      1246  24569    │  2347  12357  1234     │  13569  4579   1345679  
///   12459  124   3        │  6     12578  1248     │  1589   45789  14579
///   1456   7     456      │  348   9      1348     │  2      458    13456
/// ────────────────────────┼────────────────────────┼─────────────────────────
///   123469  5      2469   │  2389  2368  7         │  1689  2489  12469
///   12369   12368  269    │  2389  4     5         │  7     289   1269
///   24679   2468   24679  │  1     268   2689      │  5689  3     24569
/// ────────────────────────┼────────────────────────┼─────────────────────────
///   23457  234   1        │  23479  237     2349   │  359  6    8
///   23467  2346  8        │  5      2367    23469  │  39   1    2379
///   23567  9     2567     │  2378   123678  12368  │  4    257  2357
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CandidatesGridCompact;

impl GridFormat for CandidatesGridCompact {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueOnly,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::OnlyMultiple,
            detect_and_parse: GridFormatDetectAndParseCapability::Detectable,
        }
    }

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
                    block_builder.build().with(Style::empty()).to_string()
                })
            })
            .collect();

        let mut table = grid_builder.build();

        table
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
        static GRID_BORDER_CHARS: &[char] = &['-', '|', '│', ':', '+', '\'', '\n', '*'];

        input
            .lines()
            // Filter horizontal separator lines
            .filter(|&line| line.contains(|c: char| c.is_digit(36)))
            // Filter vertical separators
            .flat_map(|line| line.split(GRID_BORDER_CHARS))
            .filter(|s| !s.is_empty())
            // Split and trim groups of numbers
            .flat_map(|s| s.split_whitespace())
            .map(TryInto::<DynamicCell>::try_into)
            .collect::<Result<Vec<_>>>()?
            .try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::consts::Base3, grid::format::test_util::assert_parsed_grid, samples};
    use indoc::indoc;

    #[test]
    fn test_render() {
        let mut grid = samples::base_3().into_iter().next().unwrap();
        grid.set_all_direct_candidates();

        assert_eq!(
            CandidatesGridCompact.render(&grid),
            indoc! {"
                  8      1246  24569    │  2347  12357  1234     │  13569  4579   1345679  
                  12459  124   3        │  6     12578  1248     │  1589   45789  14579    
                  1456   7     456      │  348   9      1348     │  2      458    13456    
                ────────────────────────┼────────────────────────┼─────────────────────────
                  123469  5      2469   │  2389  2368  7         │  1689  2489  12469      
                  12369   12368  269    │  2389  4     5         │  7     289   1269       
                  24679   2468   24679  │  1     268   2689      │  5689  3     24569      
                ────────────────────────┼────────────────────────┼─────────────────────────
                  23457  234   1        │  23479  237     2349   │  359  6    8            
                  23467  2346  8        │  5      2367    23469  │  39   1    2379         
                  23567  9     2567     │  2378   123678  12368  │  4    257  2357         "
            }
        );
    }

    #[test]
    fn test_parse() {
        use crate::cell::dynamic::{c, v};

        let cells = CandidatesGridCompact
            .parse(indoc! {"
                .--------------.----------------.------------.
                | 6   7    89  | 189  19   2    | 3   5   4  |
                | 1   2    5   | .    3    4    | 9   8   7  |
                | 3   89   4   | 7    58   59   | 6   2   1  |
                :--------------+----------------+------------:
                | 7   3    29  | 19   25   1569 | 8   4   69 |
                | 5   1    289 | 89   0    679  | 27  69  3  |
                | 89  4    6   | 3    28   79   | 27  1   5  |
                :--------------+----------------+------------:
                | 2   5    3   | 4    7    8    | 0   69  69 |
                | 89  689  1   | 5    69   3    | 4   .   2  |
                | 4   69   7   | 2    169  169  | 5   3   8  |
                '--------------'----------------'------------'"
            })
            .unwrap();

        let expected_grid = Grid::<Base3>::try_from(vec![
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
        ])
        .unwrap();
        assert_parsed_grid(&expected_grid, &cells).unwrap();
    }
}
