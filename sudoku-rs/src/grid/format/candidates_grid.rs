use crate::base::SudokuBase;
use crate::base::consts::ALL_SIDE_LENGTHS;
use crate::cell::dynamic::{DynamicCandidates, DynamicCell, char_value_to_u8};
use crate::cell::{CellState, Value};
use crate::error::Result;
use crate::grid::Grid;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormatCapabilities;
use crate::grid::format::GridFormatPreservesCellCandidates;
use crate::grid::format::GridFormatPreservesCellValue;
use crate::grid::format::{GridFormat, GridFormatDetectAndParseCapability};
use anyhow::{Context, bail, ensure};
use itertools::Itertools;
use num::Integer;
use owo_colors::Style as OwoStyle;
use std::iter;
use tabled::builder::Builder;
use tabled::settings::{Padding, Style};

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
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            // Could support `ValueAndFixedState`, but parsing of ansii escape codes is not implemented.
            preserves_cell_value: GridFormatPreservesCellValue::ValueOnly,
            // The representation of a single candidate 5 in base 3 is indistinguishable from a value 5; both are a centered "5".
            preserves_cell_candidates: GridFormatPreservesCellCandidates::OnlyMultiple,
            detect_and_parse: GridFormatDetectAndParseCapability::Detectable,
        }
    }

    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        render_candidates_grid(grid, true)
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        let stripped_input = strip_ansi_escapes::strip_str(input);

        CandidatesGridPlain.parse(&stripped_input)
    }
}

/// The same as `CandidatesGridANSIStyled`, but without terminal styling.
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
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueOnly,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::OnlyMultiple,
            // Is detected as `CandidatesGridANSIStyled`, since we don't implement ANSII escape code parsing.
            detect_and_parse: GridFormatDetectAndParseCapability::DetectableViaOtherFormat,
        }
    }

    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        render_candidates_grid(grid, false)
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        fn ensure_same_line_char_count(input: &str) -> Result<usize> {
            let mut line_char_count = None;
            for line in input.lines() {
                let current_line_char_count = line.chars().count();
                if current_line_char_count == 0 {
                    bail!("Unexpected empty line")
                }
                if let Some(previous_line_char_count) = line_char_count {
                    if current_line_char_count != previous_line_char_count {
                        bail!(
                            "Expected line char count {previous_line_char_count}, instead got: {current_line_char_count}"
                        )
                    }
                } else {
                    line_char_count = Some(current_line_char_count);
                }
            }
            if let Some(line_char_count) = line_char_count {
                Ok(line_char_count)
            } else {
                bail!("Unexpected empty input")
            }
        }

        // Reference: https://stackoverflow.com/a/64499219
        fn transpose2<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
            assert!(!v.is_empty());
            let len = v[0].len();
            let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
            (0..len)
                .map(|_| {
                    iters
                        .iter_mut()
                        .map(|n| n.next().unwrap())
                        .collect::<Vec<T>>()
                })
                .collect()
        }

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

        let _line_char_count = ensure_same_line_char_count(input)?;

        // cell_str_fragments: Vec<Data for a cell row>, len() == sudoku side length
        // Data for a cell row: Vec<Single line data for cell row>, len() == cell height
        // Single line data for cell row: Vec<Single cell line fragment>, len() == cell width
        let mut cell_str_fragments: Vec<Vec<Vec<&str>>> = vec![];

        for (is_horizontal_separator, lines_with_cell_data) in &input
            .lines()
            .map(|line| line.trim_matches(OUTER_BORDER_CHARS))
            .chunk_by(|line| {
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

        let (cell_row_count_usize, cell_height, cell_column_count, cell_width) =
            if let Some(first_cell_row_data) = cell_str_fragments.first() {
                if let Some(first_line_fragments) = first_cell_row_data.first() {
                    if let Some(first_line_fragment) = first_line_fragments.first() {
                        (
                            cell_str_fragments.len(),
                            first_cell_row_data.len(),
                            first_line_fragments.len(),
                            first_line_fragment.len(),
                        )
                    } else {
                        bail!("Expected at least one cell line fragment")
                    }
                } else {
                    bail!("Expected at least one line of cell data")
                }
            } else {
                bail!("Expected at least one cell row")
            };

        ensure!(
            cell_row_count_usize == cell_column_count,
            "Expected a square sudoku grid"
        );

        let side_length_usize = cell_row_count_usize;
        let side_length = u8::try_from(side_length_usize).with_context(|| {
            format!(
                "Side length too large, expected maximum {}, instead got {side_length_usize}",
                ALL_SIDE_LENGTHS.last().unwrap()
            )
        })?;
        if !ALL_SIDE_LENGTHS.contains(&side_length) {
            bail!("Unexpected side length {side_length}, expected one of: {ALL_SIDE_LENGTHS:?}")
        }

        ensure!(cell_height % 2 == 1, "Expected cell height to be uneven");
        ensure!(cell_width % 2 == 1, "Expected cell width to be uneven");

        for cell_row_data in &cell_str_fragments {
            ensure!(
                cell_row_data.len() == cell_height,
                "Expected cell height to be consistent"
            );

            for line_fragments in cell_row_data {
                ensure!(
                    line_fragments.len() == side_length_usize,
                    "Expected cell column count to be consistent"
                );

                for line_fragment in line_fragments {
                    ensure!(
                        line_fragment.len() == cell_width,
                        "Expected cell width to be consistent"
                    );
                }
            }
        }

        let cell_str_fragments_transposed: Vec<_> =
            cell_str_fragments.into_iter().map(transpose2).collect();

        let dynamic_cells = cell_str_fragments_transposed
            .into_iter()
            .flatten()
            .map(|cell_fragments| {
                debug_assert!(cell_fragments.len() == cell_height);

                let single_centered_char = {
                    let (first_fragments, middle_and_last_fragments) =
                        cell_fragments.split_at(cell_height / 2);
                    let (middle_fragment, last_fragments) =
                        middle_and_last_fragments.split_first().unwrap();

                    if first_fragments.iter().all(|s| s.trim().is_empty())
                        && last_fragments.iter().all(|s| s.trim().is_empty())
                    {
                        let middle_fragment_trimmed = middle_fragment.trim();
                        (middle_fragment_trimmed.len() == 1
                            && middle_fragment.find(middle_fragment_trimmed).unwrap()
                                == cell_width / 2)
                            .then(|| middle_fragment_trimmed.chars().next().unwrap())
                    } else {
                        None
                    }
                };

                if let Some(single_centered_char) = single_centered_char {
                    DynamicCell::try_from(single_centered_char)
                } else {
                    Ok(DynamicCandidates::from(
                        cell_fragments
                            .into_iter()
                            .flat_map(|cell_fragment| cell_fragment.trim().chars())
                            .filter(|c| !c.is_whitespace())
                            .map(char_value_to_u8)
                            .collect::<Result<Vec<_>>>()?,
                    )
                    .into())
                }
            })
            .collect::<Result<Vec<_>>>()?;

        dynamic_cells.try_into()
    }
}

fn render_candidates_grid<Base: SudokuBase>(
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
                                value_table_builder
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
                                    .to_string()
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
            let dynamic_cells = CandidatesGridPlain
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

            dbg!(dynamic_cells);
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
            let grid = samples::base_3().into_iter().next().unwrap();
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
        mod snapshots {
            use super::*;

            #[test]
            fn test_render_compact_base3() {
                for (i, grid) in samples::base_3().into_iter().enumerate() {
                    insta::assert_snapshot!(
                        format!("candidates_plain_compact_base3_{i}"),
                        CandidatesGridPlain.render(&grid)
                    );
                }
            }

            #[test]
            fn test_render_with_candidates_base3() {
                for (i, mut grid) in samples::base_3().into_iter().enumerate() {
                    grid.set_all_direct_candidates();
                    insta::assert_snapshot!(
                        format!("candidates_plain_base3_{i}"),
                        CandidatesGridPlain.render(&grid)
                    );
                }
            }
        }

        #[test]
        fn test_render_base_3_sparse() {
            let mut grid = samples::base_3().into_iter().next().unwrap();
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
