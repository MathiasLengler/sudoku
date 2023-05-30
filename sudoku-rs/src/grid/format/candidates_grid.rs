use std::iter;

use anyhow::bail;
use itertools::Itertools;
use num::Integer;
use owo_colors::Style as OwoStyle;
use tabled::builder::Builder;
use tabled::settings::{Padding, Style};

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::{CellState, Value};
use crate::error::Result;
use crate::grid::format::GridFormat;
use crate::grid::Grid;

/// A grid of cells.
/// Values are centered.
/// Candidates are visualized as a nested grid, which spans multiple lines.
/// If the grid contains no set candidates, the grid is rendered compactly.
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

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>> {
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
/// # Examples
///
/// ## Base 2
/// No candidates:
/// ```text
/// ╔═══════╦═══════╗
/// ║   │   ║ 1 │   ║
/// ║───┼───║───┼───║
/// ║ 4 │   ║   │   ║
/// ╠═══════╬═══════╣
/// ║   │   ║   │ 2 ║
/// ║───┼───║───┼───║
/// ║   │ 3 ║   │   ║
/// ╚═══════╩═══════╝
/// ```
/// With candidates:
/// ```text
/// ╔═══════════╦═══════════╗
/// ║     │     ║     │     ║
/// ║     │  2  ║  1  │     ║
/// ║ 3   │     ║     │ 3 4 ║
/// ║─────┼─────║─────┼─────║
/// ║     │ 1   ║   2 │     ║
/// ║  4  │     ║     │     ║
/// ║     │     ║ 3   │ 3   ║
/// ╠═══════════╬═══════════╣
/// ║ 1   │ 1   ║     │     ║
/// ║     │     ║     │  2  ║
/// ║     │   4 ║ 3 4 │     ║
/// ║─────┼─────║─────┼─────║
/// ║ 1 2 │     ║     │ 1   ║
/// ║     │  3  ║     │     ║
/// ║     │     ║   4 │   4 ║
/// ╚═══════════╩═══════════╝
/// ```
/// ## Base 3
/// No candidates:
/// ```text
/// ╔═══════════╦═══════════╦═══════════╗
/// ║ 8 │   │   ║   │   │   ║   │   │   ║
/// ║───┼───┼───║───┼───┼───║───┼───┼───║
/// ║   │   │ 3 ║ 6 │   │   ║   │   │   ║
/// ║───┼───┼───║───┼───┼───║───┼───┼───║
/// ║   │ 7 │   ║   │ 9 │   ║ 2 │   │   ║
/// ╠═══════════╬═══════════╬═══════════╣
/// ║   │ 5 │   ║   │   │ 7 ║   │   │   ║
/// ║───┼───┼───║───┼───┼───║───┼───┼───║
/// ║   │   │   ║   │ 4 │ 5 ║ 7 │   │   ║
/// ║───┼───┼───║───┼───┼───║───┼───┼───║
/// ║   │   │   ║ 1 │   │   ║   │ 3 │   ║
/// ╠═══════════╬═══════════╬═══════════╣
/// ║   │   │ 1 ║   │   │   ║   │ 6 │ 8 ║
/// ║───┼───┼───║───┼───┼───║───┼───┼───║
/// ║   │   │ 8 ║ 5 │   │   ║   │ 1 │   ║
/// ║───┼───┼───║───┼───┼───║───┼───┼───║
/// ║   │ 9 │   ║   │   │   ║ 4 │   │   ║
/// ╚═══════════╩═══════════╩═══════════╝
/// ```
/// With candidates:
/// ```text
/// ╔═════════════════╦═════════════════╦═════════════════╗
/// ║     │ 12  │  2  ║  23 │ 123 │ 123 ║ 1 3 │     │ 1 3 ║
/// ║  8  │ 4 6 │ 456 ║ 4   │  5  │ 4   ║  56 │ 45  │ 456 ║
/// ║     │     │   9 ║ 7   │ 7   │     ║   9 │ 7 9 │ 7 9 ║
/// ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
/// ║ 12  │ 12  │     ║     │ 12  │ 12  ║ 1   │     │ 1   ║
/// ║ 45  │ 4   │  3  ║  6  │  5  │ 4   ║  5  │ 45  │ 45  ║
/// ║   9 │     │     ║     │ 78  │  8  ║  89 │ 789 │ 7 9 ║
/// ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
/// ║ 1   │     │     ║   3 │     │ 1 3 ║     │     │ 1 3 ║
/// ║ 456 │  7  │ 456 ║ 4   │  9  │ 4   ║  2  │ 45  │ 456 ║
/// ║     │     │     ║  8  │     │  8  ║     │  8  │     ║
/// ╠═════════════════╬═════════════════╬═════════════════╣
/// ║ 123 │     │  2  ║  23 │  23 │     ║ 1   │  2  │ 12  ║
/// ║ 4 6 │  5  │ 4 6 ║     │   6 │  7  ║   6 │ 4   │ 4 6 ║
/// ║   9 │     │   9 ║  89 │  8  │     ║  89 │  89 │   9 ║
/// ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
/// ║ 123 │ 123 │  2  ║  23 │     │     ║     │  2  │ 12  ║
/// ║   6 │   6 │   6 ║     │  4  │  5  ║  7  │     │   6 ║
/// ║   9 │  8  │   9 ║  89 │     │     ║     │  89 │   9 ║
/// ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
/// ║  2  │  2  │  2  ║     │  2  │  2  ║     │     │  2  ║
/// ║ 4 6 │ 4 6 │ 4 6 ║  1  │   6 │   6 ║  56 │  3  │ 456 ║
/// ║ 7 9 │  8  │ 7 9 ║     │  8  │  89 ║  89 │     │   9 ║
/// ╠═════════════════╬═════════════════╬═════════════════╣
/// ║  23 │  23 │     ║  23 │  23 │  23 ║   3 │     │     ║
/// ║ 45  │ 4   │  1  ║ 4   │     │ 4   ║  5  │  6  │  8  ║
/// ║ 7   │     │     ║ 7 9 │ 7   │   9 ║   9 │     │     ║
/// ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
/// ║  23 │  23 │     ║     │  23 │  23 ║   3 │     │  23 ║
/// ║ 4 6 │ 4 6 │  8  ║  5  │   6 │ 4 6 ║     │  1  │     ║
/// ║ 7   │     │     ║     │ 7   │   9 ║   9 │     │ 7 9 ║
/// ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
/// ║  23 │     │  2  ║  23 │ 123 │ 123 ║     │  2  │  23 ║
/// ║  56 │  9  │  56 ║     │   6 │   6 ║  4  │  5  │  5  ║
/// ║ 7   │     │ 7   ║ 78  │ 78  │  8  ║     │ 7   │ 7   ║
/// ╚═════════════════╩═════════════════╩═════════════════╝
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CandidatesGridPlain;

impl GridFormat for CandidatesGridPlain {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        render_candidates_grid(grid, false)
    }

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>> {
        // TODO: implement
        //  split into multi-line rows
        //  split rows into multi-line cells
        //  extract numbers from cells

        fn ensure_same_line_char_count(input: &str) -> Result<usize> {
            let mut line_char_count = None;
            for line in input.lines() {
                let current_line_char_count = line.chars().count();
                if current_line_char_count == 0 {
                    bail!("Unexpected empty line")
                }
                if let Some(previous_line_char_count) = line_char_count {
                    if current_line_char_count != previous_line_char_count {
                        bail!("Expected line char count {previous_line_char_count}, instead got: {current_line_char_count}")
                    }
                } else {
                    line_char_count = Some(current_line_char_count)
                }
            }
            if let Some(line_char_count) = line_char_count {
                Ok(line_char_count)
            } else {
                bail!("Unexpected empty input")
            }
        }

        println!("input {input}");

        const FIRST_CHAR: char = '╔';
        const OUTER_BORDER_CHARS: &[char] = &[
            '║', '═', // Straight
            '╔', '╦', '╗', // Top
            '╠', '╬', '╣', // Middle
            '╚', '╩', '╝', // Bottom
        ];

        const INNER_BORDER_CHARS: &[char] = &['─', '│', '┼'];
        const VERTICAL_BORDER_CHARS: &[char] = &['│', '║'];

        match input.chars().next() {
            Some(char) if char == FIRST_CHAR => {}
            Some(unexpected_char) => {
                bail!("Expected first character to be {FIRST_CHAR}, instead got: {unexpected_char}")
            }
            None => bail!("Unexpected empty input"),
        }

        let line_char_count = ensure_same_line_char_count(input)?;

        // cell_str_fragments: Vec<Data for a cell row>, len() == sudoku side length
        // Data for a cell row: Vec<Single line data for cell row>, len() == cell height
        // Single line data for cell row: Vec<Single cell line fragment>, len() == cell width
        let mut cell_str_fragments: Vec<Vec<Vec<&str>>> = vec![];

        for (is_horizontal_separator, lines_with_cell_data) in &input
            .lines()
            .map(|line| line.trim_matches(OUTER_BORDER_CHARS))
            .group_by(|line| {
                line.is_empty()
                    || line.chars().all(|char| {
                        OUTER_BORDER_CHARS.contains(&char) || INNER_BORDER_CHARS.contains(&char)
                    })
            })
        {
            if !is_horizontal_separator {
                cell_str_fragments.push(
                    lines_with_cell_data
                        .map(|line_with_cell_data| {
                            line_with_cell_data
                                .split(VERTICAL_BORDER_CHARS)
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }

        dbg!(cell_str_fragments);

        bail!("todo: process cell_str_fragments")
    }
}

pub fn render_candidates_grid<Base: SudokuBase>(
    grid: &Grid<Base>,
    enable_terminal_styling: bool,
) -> String {
    let bold;
    let bold_blue;
    if enable_terminal_styling {
        bold = OwoStyle::new().bold();
        bold_blue = OwoStyle::new().bold().blue();
    } else {
        bold = OwoStyle::new();
        bold_blue = OwoStyle::new();
    }

    let is_compact = !grid.all_cells().any(
        |cell| matches!(cell.state(), CellState::Candidates(candidates) if !candidates.is_empty()),
    );

    let is_even_base = Base::BASE.is_even();

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
                        block_row.iter().map(|cell| match cell.state() {
                            CellState::Value(value) | CellState::FixedValue(value) => {
                                let value_string = value.to_string();
                                let value_string_colored = if cell.has_fixed_value() {
                                    bold.style(value_string)
                                } else {
                                    bold_blue.style(value_string)
                                }
                                .to_string();
                                let value_table_builder: Builder =
                                    iter::once(iter::once(value_string_colored)).collect();
                                let value_string_with_padding = value_table_builder
                                    .build()
                                    .with(if is_compact {
                                        Padding::zero()
                                    } else {
                                        let padding = usize::from(if is_even_base {
                                            Base::BASE - 1
                                        } else {
                                            Base::BASE - 2
                                        });
                                        Padding::new(padding, padding, padding, padding)
                                    })
                                    .with(Style::empty())
                                    .to_string();
                                value_string_with_padding
                            }
                            CellState::Candidates(candidates) => {
                                if is_compact {
                                    " ".to_string()
                                } else {
                                    let candidates_builder: Builder = all_values
                                        .chunks(usize::from(Base::BASE))
                                        .map(|all_candidates_row| {
                                            all_candidates_row.iter().map(|candidate| {
                                                if candidates.has(*candidate) {
                                                    candidate.to_string()
                                                } else {
                                                    " ".to_string()
                                                }
                                            })
                                        })
                                        .collect();

                                    let mut candidates_table = candidates_builder.build();
                                    candidates_table.with(Padding::zero());
                                    if is_even_base {
                                        candidates_table
                                            .with(Style::empty().vertical(' ').horizontal(' '));
                                    } else {
                                        candidates_table.with(Style::empty());
                                    }

                                    candidates_table.to_string()
                                }
                            }
                        })
                    })
                    .collect();
                block_builder
                    .build()
                    .with(Padding::new(1, 1, 0, 0))
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
        .with(Padding::zero())
        .with(Style::extended())
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;
    use crate::samples;

    use super::*;

    fn base_2_sparse_grid() -> Grid<Base2> {
        let mut grid = samples::base_2().pop().unwrap();
        grid.fix_all_values();
        grid.get_mut((0, 1).try_into().unwrap())
            .set_value(2.try_into().unwrap());
        grid.set_all_direct_candidates();
        grid
    }

    mod ansi_styled {
        use super::*;

        #[test]
        fn test_render_base_2_sparse() {
            let grid = base_2_sparse_grid();
            assert_eq!(
                CandidatesGridANSIStyled.render(&grid),
                "╔═══════════╦═══════════╗
║     │     ║     │     ║
║     │  \u{1b}[34;1m2\u{1b}[0m  ║  \u{1b}[1m1\u{1b}[0m  │     ║
║ 3   │     ║     │ 3 4 ║
║─────┼─────║─────┼─────║
║     │ 1   ║   2 │     ║
║  \u{1b}[1m4\u{1b}[0m  │     ║     │     ║
║     │     ║ 3   │ 3   ║
╠═══════════╬═══════════╣
║ 1   │ 1   ║     │     ║
║     │     ║     │  \u{1b}[1m2\u{1b}[0m  ║
║     │   4 ║ 3 4 │     ║
║─────┼─────║─────┼─────║
║ 1 2 │     ║     │ 1   ║
║     │  \u{1b}[1m3\u{1b}[0m  ║     │     ║
║     │     ║   4 │   4 ║
╚═══════════╩═══════════╝"
            );
        }
    }

    mod plain {
        use super::*;

        #[test]
        fn test_parse_base_2_compact() {
            CandidatesGridPlain
                .parse(
                    "╔═══════╦═══════╗
║   │   ║ 1 │   ║
║───┼───║───┼───║
║ 4 │   ║   │   ║
╠═══════╬═══════╣
║   │   ║   │ 2 ║
║───┼───║───┼───║
║   │ 3 ║   │   ║
╚═══════╩═══════╝",
                )
                .unwrap();
        }
        #[test]
        fn test_render_base_2_compact() {
            let grid = samples::base_2().pop().unwrap();
            assert_eq!(
                CandidatesGridPlain.render(&grid),
                "╔═══════╦═══════╗
║   │   ║ 1 │   ║
║───┼───║───┼───║
║ 4 │   ║   │   ║
╠═══════╬═══════╣
║   │   ║   │ 2 ║
║───┼───║───┼───║
║   │ 3 ║   │   ║
╚═══════╩═══════╝"
            );
        }

        #[test]
        fn test_parse_base_2_sparse() {
            CandidatesGridPlain
                .parse(
                    "╔═══════════╦═══════════╗
║     │     ║     │     ║
║     │  2  ║  1  │     ║
║ 3   │     ║     │ 3 4 ║
║─────┼─────║─────┼─────║
║     │ 1   ║   2 │     ║
║  4  │     ║     │     ║
║     │     ║ 3   │ 3   ║
╠═══════════╬═══════════╣
║ 1   │ 1   ║     │     ║
║     │     ║     │  2  ║
║     │   4 ║ 3 4 │     ║
║─────┼─────║─────┼─────║
║ 1 2 │     ║     │ 1   ║
║     │  3  ║     │     ║
║     │     ║   4 │   4 ║
╚═══════════╩═══════════╝",
                )
                .unwrap();
        }

        #[test]
        fn test_render_base_2_sparse() {
            let grid = base_2_sparse_grid();

            assert_eq!(
                CandidatesGridPlain.render(&grid),
                "╔═══════════╦═══════════╗
║     │     ║     │     ║
║     │  2  ║  1  │     ║
║ 3   │     ║     │ 3 4 ║
║─────┼─────║─────┼─────║
║     │ 1   ║   2 │     ║
║  4  │     ║     │     ║
║     │     ║ 3   │ 3   ║
╠═══════════╬═══════════╣
║ 1   │ 1   ║     │     ║
║     │     ║     │  2  ║
║     │   4 ║ 3 4 │     ║
║─────┼─────║─────┼─────║
║ 1 2 │     ║     │ 1   ║
║     │  3  ║     │     ║
║     │     ║   4 │   4 ║
╚═══════════╩═══════════╝"
            );
        }

        #[test]
        fn test_render_base_3_compact() {
            let grid = samples::base_3().pop().unwrap();
            assert_eq!(
                CandidatesGridPlain.render(&grid),
                "╔═══════════╦═══════════╦═══════════╗
║ 8 │   │   ║   │   │   ║   │   │   ║
║───┼───┼───║───┼───┼───║───┼───┼───║
║   │   │ 3 ║ 6 │   │   ║   │   │   ║
║───┼───┼───║───┼───┼───║───┼───┼───║
║   │ 7 │   ║   │ 9 │   ║ 2 │   │   ║
╠═══════════╬═══════════╬═══════════╣
║   │ 5 │   ║   │   │ 7 ║   │   │   ║
║───┼───┼───║───┼───┼───║───┼───┼───║
║   │   │   ║   │ 4 │ 5 ║ 7 │   │   ║
║───┼───┼───║───┼───┼───║───┼───┼───║
║   │   │   ║ 1 │   │   ║   │ 3 │   ║
╠═══════════╬═══════════╬═══════════╣
║   │   │ 1 ║   │   │   ║   │ 6 │ 8 ║
║───┼───┼───║───┼───┼───║───┼───┼───║
║   │   │ 8 ║ 5 │   │   ║   │ 1 │   ║
║───┼───┼───║───┼───┼───║───┼───┼───║
║   │ 9 │   ║   │   │   ║ 4 │   │   ║
╚═══════════╩═══════════╩═══════════╝"
            );
        }
        #[test]
        fn test_render_base_3_sparse() {
            let mut grid = samples::base_3().pop().unwrap();
            grid.set_all_direct_candidates();
            assert_eq!(
                CandidatesGridPlain.render(&grid),
                "╔═════════════════╦═════════════════╦═════════════════╗
║     │ 12  │  2  ║  23 │ 123 │ 123 ║ 1 3 │     │ 1 3 ║
║  8  │ 4 6 │ 456 ║ 4   │  5  │ 4   ║  56 │ 45  │ 456 ║
║     │     │   9 ║ 7   │ 7   │     ║   9 │ 7 9 │ 7 9 ║
║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
║ 12  │ 12  │     ║     │ 12  │ 12  ║ 1   │     │ 1   ║
║ 45  │ 4   │  3  ║  6  │  5  │ 4   ║  5  │ 45  │ 45  ║
║   9 │     │     ║     │ 78  │  8  ║  89 │ 789 │ 7 9 ║
║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
║ 1   │     │     ║   3 │     │ 1 3 ║     │     │ 1 3 ║
║ 456 │  7  │ 456 ║ 4   │  9  │ 4   ║  2  │ 45  │ 456 ║
║     │     │     ║  8  │     │  8  ║     │  8  │     ║
╠═════════════════╬═════════════════╬═════════════════╣
║ 123 │     │  2  ║  23 │  23 │     ║ 1   │  2  │ 12  ║
║ 4 6 │  5  │ 4 6 ║     │   6 │  7  ║   6 │ 4   │ 4 6 ║
║   9 │     │   9 ║  89 │  8  │     ║  89 │  89 │   9 ║
║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
║ 123 │ 123 │  2  ║  23 │     │     ║     │  2  │ 12  ║
║   6 │   6 │   6 ║     │  4  │  5  ║  7  │     │   6 ║
║   9 │  8  │   9 ║  89 │     │     ║     │  89 │   9 ║
║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
║  2  │  2  │  2  ║     │  2  │  2  ║     │     │  2  ║
║ 4 6 │ 4 6 │ 4 6 ║  1  │   6 │   6 ║  56 │  3  │ 456 ║
║ 7 9 │  8  │ 7 9 ║     │  8  │  89 ║  89 │     │   9 ║
╠═════════════════╬═════════════════╬═════════════════╣
║  23 │  23 │     ║  23 │  23 │  23 ║   3 │     │     ║
║ 45  │ 4   │  1  ║ 4   │     │ 4   ║  5  │  6  │  8  ║
║ 7   │     │     ║ 7 9 │ 7   │   9 ║   9 │     │     ║
║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
║  23 │  23 │     ║     │  23 │  23 ║   3 │     │  23 ║
║ 4 6 │ 4 6 │  8  ║  5  │   6 │ 4 6 ║     │  1  │     ║
║ 7   │     │     ║     │ 7   │   9 ║   9 │     │ 7 9 ║
║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
║  23 │     │  2  ║  23 │ 123 │ 123 ║     │  2  │  23 ║
║  56 │  9  │  56 ║     │   6 │   6 ║  4  │  5  │  5  ║
║ 7   │     │ 7   ║ 78  │ 78  │  8  ║     │ 7   │ 7   ║
╚═════════════════╩═════════════════╩═════════════════╝"
            );
        }
    }
}
