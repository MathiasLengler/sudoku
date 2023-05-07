use anyhow::bail;
use owo_colors::Style as OwoStyle;
use tabled::builder::Builder;
use tabled::settings::{object::Segment, Alignment, Modify, Style};

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::{CellState, Value};
use crate::error;
use crate::grid::format::GridFormat;
use crate::grid::Grid;

/// A grid of cells.
/// Candidates are visualized as a nested grid, which spans multiple lines.
/// The grid borders are represented by [UTF-8 box drawing characters](https://en.wikipedia.org/wiki/Box_Drawing).
///
/// Cell content is styled with [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code).
///
/// - unfixed value: bold blue
/// - fixed value: bold
/// - candidates: default
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CandidatesGridANSIStyled;

impl GridFormat for CandidatesGridANSIStyled {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        render_candidates_grid(grid, true)
    }

    fn parse(self, input: &str) -> error::Result<Vec<DynamicCell>> {
        let stripped_input_bytes = strip_ansi_escapes::strip(input.as_bytes())?;
        let stripped_input = String::from_utf8(stripped_input_bytes)?;

        CandidatesGridPlain.parse(&stripped_input)
    }

    fn do_fix_all_values(self) -> bool {
        false
    }
}

/// The same as `CandidatesGridColored`, but without terminal styling.
///
/// # Example
///
/// ```text
/// ╔═══════════════════╦═══════════════════╗
/// ║         │         ║         │         ║
/// ║         │   2     ║    1    │         ║
/// ║   3     │         ║         │  3  4   ║
/// ║ ────────┼──────── ║ ────────┼──────── ║
/// ║         │  1      ║      2  │         ║
/// ║    4    │         ║         │         ║
/// ║         │         ║   3     │  3      ║
/// ╠═══════════════════╬═══════════════════╣
/// ║   1     │  1      ║         │         ║
/// ║         │         ║         │   2     ║
/// ║         │     4   ║   3  4  │         ║
/// ║ ────────┼──────── ║ ────────┼──────── ║
/// ║   1  2  │         ║         │  1      ║
/// ║         │   3     ║         │         ║
/// ║         │         ║      4  │     4   ║
/// ╚═══════════════════╩═══════════════════╝
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CandidatesGridPlain;

impl GridFormat for CandidatesGridPlain {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        render_candidates_grid(grid, false)
    }

    fn parse(self, _input: &str) -> error::Result<Vec<DynamicCell>> {
        // TODO: implement
        //  split into multi-line rows
        //  split rows into multi-line cells
        //  extract numbers from cells

        bail!("todo")
    }
}

pub fn render_candidates_grid<Base: SudokuBase>(
    grid: &Grid<Base>,
    enable_terminal_styling: bool,
) -> String {
    let default;
    let bold;
    let bold_blue;
    if enable_terminal_styling {
        default = OwoStyle::new();
        bold = OwoStyle::new().bold();
        bold_blue = OwoStyle::new().bold().blue();
    } else {
        default = OwoStyle::new();
        bold = OwoStyle::new();
        bold_blue = OwoStyle::new();
    }

    let all_values: Vec<_> = (1..=Base::MAX_VALUE)
        .map(|value| Value::<Base>::new(value).unwrap().unwrap())
        .collect();

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
                        block_row
                            .iter()
                            .map(|cell| match cell.state() {
                                CellState::Value(value) => bold_blue.style(value.to_string()),
                                CellState::FixedValue(value) => bold.style(value.to_string()),
                                CellState::Candidates(candidates) => {
                                    let mut candidates_builder = Builder::new();

                                    all_values.chunks(usize::from(Base::BASE)).for_each(
                                        |all_candidates_row| {
                                            candidates_builder.push_record(
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
                                    default.style(
                                        candidates_builder
                                            .build()
                                            .with(Style::empty().horizontal(' '))
                                            .to_string(),
                                    )
                                }
                            })
                            .map(|styled_s| styled_s.to_string())
                    })
                    .collect();
                block_builder
                    .build()
                    .with(
                        Modify::new(Segment::all())
                            .with(Alignment::center())
                            .with(Alignment::center_vertical()),
                    )
                    .with(
                        Style::modern()
                            .remove_top()
                            .remove_left()
                            .remove_right()
                            .remove_bottom(),
                    )
                    .to_string()
            })
        })
        .collect();

    let mut table = grid_builder.build();

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
    use super::*;
    use crate::samples;

    mod ansi_styled {
        use super::*;
        #[test]
        fn test_render() {
            let mut grid = samples::base_2().pop().unwrap();
            grid.fix_all_values();
            grid.get_mut((0, 1).try_into().unwrap())
                .set_value(2.try_into().unwrap());
            grid.set_all_direct_candidates();

            assert_eq!(
                CandidatesGridANSIStyled.render(&grid),
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

    mod plain {
        use super::*;

        #[test]
        fn test_render() {
            let mut grid = samples::base_2().pop().unwrap();
            grid.fix_all_values();
            grid.get_mut((0, 1).try_into().unwrap())
                .set_value(2.try_into().unwrap());
            grid.set_all_direct_candidates();

            assert_eq!(
                CandidatesGridPlain.render(&grid),
                "╔═══════════════════╦═══════════════════╗
║         │         ║         │         ║
║         │   2     ║    1    │         ║
║   3     │         ║         │  3  4   ║
║ ────────┼──────── ║ ────────┼──────── ║
║         │  1      ║      2  │         ║
║    4    │         ║         │         ║
║         │         ║   3     │  3      ║
╠═══════════════════╬═══════════════════╣
║   1     │  1      ║         │         ║
║         │         ║         │   2     ║
║         │     4   ║   3  4  │         ║
║ ────────┼──────── ║ ────────┼──────── ║
║   1  2  │         ║         │  1      ║
║         │   3     ║         │         ║
║         │         ║      4  │     4   ║
╚═══════════════════╩═══════════════════╝"
            );
        }
    }
}
