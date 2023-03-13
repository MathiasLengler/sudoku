use owo_colors::Style as OwoStyle;
use serde::{Deserialize, Serialize};
use tabled::{builder::Builder, object::Segment, Alignment, Modify, Style};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::compact::cell_state::CellState;
use crate::cell::compact::value::Value;
use crate::grid::Grid;

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GridFormat {
    /// # Example
    /// `800000000003600000070090200050007000000045700000100030001000068008500010090000400`
    GivensLine,
    /// # Example
    /// ```text
    ///   8  0  0|  0  0  0|  0  0  0
    ///   0  0  3|  6  0  0|  0  0  0
    ///   0  7  0|  0  9  0|  2  0  0
    /// ------------------------------
    ///   0  5  0|  0  0  7|  0  0  0
    ///   0  0  0|  0  4  5|  7  0  0
    ///   0  0  0|  1  0  0|  0  3  0
    /// ------------------------------
    ///   0  0  1|  0  0  0|  0  6  8
    ///   0  0  8|  5  0  0|  0  1  0
    ///   0  9  0|  0  0  0|  4  0  0
    /// ```
    GivensGrid,
    /// Compact candidates grid format used by [sudokuwiki.org](https://www.sudokuwiki.org/sudoku.htm)
    /// for the search parameter "n".
    ///
    /// # Encoding
    /// - Values are encoded as `2^value`.
    /// - Candidates are encoded as a bitfield and serialized in base 10.
    /// - Empty cells are `0`.
    ///
    /// Each number is separated by a ",".
    ///
    /// # Note
    /// This format does not differentiate between single candidates and values.
    ///
    /// # Example
    /// `1,2,4,8,16,32,64,128,256,3,5,9,17,33,65,129,257,511,3,7,15,31,63,127,255,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511,511`
    BinaryCandidatesLine,
    /// Render grid of cells.
    /// Candidates are visualized as a nested grid.
    /// Values are bold.
    /// Unfixed values are blue.
    CandidatesGrid,
}

impl GridFormat {
    pub fn render<Base: SudokuBase>(&self, grid: &Grid<Base>) -> String {
        use GridFormat::*;
        match self {
            GivensLine => render_givens_line(grid),
            GivensGrid => render_givens_grid(grid),
            BinaryCandidatesLine => render_binary_candidates_line(grid),
            CandidatesGrid => render_candidates_grid(grid),
        }
    }
}

fn render_givens_line<Base: SudokuBase>(grid: &Grid<Base>) -> String {
    grid.cells.iter().map(ToString::to_string).collect()
}

fn render_givens_grid<Base: SudokuBase>(grid: &Grid<Base>) -> String {
    use itertools::Itertools;

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

    return builder
        .build()
        .with(
            Style::modern()
                .off_top()
                .off_left()
                .off_right()
                .off_bottom(),
        )
        .to_string();
}

fn render_binary_candidates_line<Base: SudokuBase>(grid: &Grid<Base>) -> String {
    use itertools::Itertools;

    grid.cells
        .iter()
        .map(|cell| match cell.state() {
            CellState::Value(value) | CellState::FixedValue(value) => {
                2usize.pow(u32::from(value.into_u8() - 1)).to_string()
            }
            CellState::Candidates(candidates) => candidates.integral().to_string(),
        })
        .join(",")
}

fn render_candidates_grid<Base: SudokuBase>(grid: &Grid<Base>) -> String {
    let default = OwoStyle::new();
    let bold = OwoStyle::new().bold();
    let bold_blue = OwoStyle::new().bold().blue();

    let all_values: Vec<_> = (1..=Base::MAX_VALUE)
        .map(|value| Value::<Base>::new(value).unwrap().unwrap())
        .collect();

    let all_block_cells = grid
        .all_block_cells()
        .map(|block| block.collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let mut grid_builder: Builder = all_block_cells
        .chunks(usize::from(Base::BASE))
        .map(|row_of_blocks| {
            row_of_blocks.into_iter().map(|block| {
                let mut block_builder: Builder = block
                    .chunks(usize::from(Base::BASE))
                    .map(|block_row| {
                        block_row.iter().map(|cell| match cell.state() {
                            CellState::Value(value) => bold_blue.style(value.to_string()),
                            CellState::FixedValue(value) => bold.style(value.to_string()),
                            CellState::Candidates(candidates) => {
                                let mut candidates_builder = Builder::new();

                                all_values.chunks(usize::from(Base::BASE)).for_each(
                                    |all_candidates_row| {
                                        candidates_builder.add_record(
                                            all_candidates_row.iter().map(|candidate| {
                                                if candidates.has(*candidate) {
                                                    candidate.to_string()
                                                } else {
                                                    " ".to_string()
                                                }
                                            }),
                                        );
                                    },
                                );
                                candidates_builder.remove_columns();
                                default.style(
                                    candidates_builder
                                        .build()
                                        .with(Style::empty().horizontal(' '))
                                        .to_string(),
                                )
                            }
                        })
                    })
                    .collect();
                block_builder.remove_columns();
                block_builder
                    .build()
                    .with(
                        Modify::new(Segment::all())
                            .with(Alignment::center())
                            .with(Alignment::center_vertical()),
                    )
                    .with(
                        Style::modern()
                            .off_top()
                            .off_left()
                            .off_right()
                            .off_bottom(),
                    )
                    .to_string()
            })
        })
        .collect();

    grid_builder.remove_columns();

    let table = grid_builder.build();

    table
        .with(
            Modify::new(Segment::all())
                .with(Alignment::center())
                .with(Alignment::center_vertical()),
        )
        .with(Style::extended())
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::position::Position;
    use crate::samples::{base_2, base_3};

    use super::*;

    #[test]
    fn test_render_givens_line() {
        let grid = base_3().pop().unwrap();

        assert_eq!(
            GridFormat::GivensLine.render(&grid),
            "800000000003600000070090200050007000000045700000100030001000068008500010090000400"
        );
    }
    #[test]
    fn test_render_givens_grid() {
        let grid = base_3().pop().unwrap();

        assert_eq!(
            GridFormat::GivensGrid.render(&grid),
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
    fn test_render_binary_candidates_line() {
        let mut grid = base_3().pop().unwrap();
        grid.set_all_direct_candidates();

        assert_eq!(
            GridFormat::BinaryCandidatesLine.render(&grid),
            "128,43,314,78,87,15,309,344,381,283,11,4,32,211,139,401,472,345,57,64,56,140,256,141,2,152,61,303,16,298,390,166,64,417,394,299,295,167,290,390,8,16,64,386,291,362,170,362,1,162,418,432,4,314,94,14,1,334,70,270,276,32,128,110,46,128,16,102,302,260,1,326,118,256,114,198,231,167,8,82,86"
        );
    }

    #[test]
    fn test_render_candidates_grid() {
        let mut grid = base_2().pop().unwrap();
        grid.fix_all_values();
        grid.get_mut(Position { row: 0, column: 1 })
            .set_value(2.try_into().unwrap());
        grid.set_all_direct_candidates();

        assert_eq!(
            GridFormat::CandidatesGrid.render(&grid),
            "╔═══════════════════╦═══════════════════╗
║         │         ║         │         ║
║         │   \u{1b}[34;1m2\u{1b}[0m     ║    \u{1b}[1m1\u{1b}[0m    │         ║
║   3     │         ║         │  3  4   ║
║ ────────┼──────── ║ ────────┼──────── ║
║         │  1      ║      2  │         ║
║    \u{1b}[1m4\u{1b}[0m    │         ║         │         ║
║         │         ║   3     │  3      ║
╠═══════════════════╬═══════════════════╣
║   1     │  1      ║         │         ║
║         │         ║         │   \u{1b}[1m2\u{1b}[0m     ║
║         │     4   ║   3  4  │         ║
║ ────────┼──────── ║ ────────┼──────── ║
║   1  2  │         ║         │  1      ║
║         │   \u{1b}[1m3\u{1b}[0m     ║         │         ║
║         │         ║      4  │     4   ║
╚═══════════════════╩═══════════════════╝"
        );
    }
}
