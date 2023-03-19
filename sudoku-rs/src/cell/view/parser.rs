use std::convert::TryInto;

use anyhow::ensure;

use crate::base::consts::ALL_CELL_COUNTS;
use crate::cell::view::CellView;
use crate::error::Result;

pub(crate) fn parse_cells(input: &str) -> Result<Vec<CellView>> {
    let input = input.trim();

    let mut cell_views = if input.contains('\n') {
        from_ascii_candidates_grid(input).unwrap_or_else(|_| from_givens_grid(input))
    } else {
        from_givens_line(input).or_else(|_| from_binary_candidates_line(input))?
    };

    let actual_cell_count = cell_views.len().try_into()?;

    ensure!(
        ALL_CELL_COUNTS.contains(&actual_cell_count),
        "Unexpected cell count {actual_cell_count}, expected one of: {ALL_CELL_COUNTS:?}"
    );

    // Fix all values
    for cell_view in &mut cell_views {
        if let CellView::Value { fixed, value } = cell_view {
            if *value != 0 {
                *fixed = true;
            }
        }
    }

    Ok(cell_views)
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

fn from_binary_candidates_line(input: &str) -> Result<Vec<CellView>> {
    let mut cell_views = vec![];

    for cell_str in input.split(',') {
        let bits = cell_str.parse::<u32>()?;
        cell_views.push(bits.try_into()?)
    }

    Ok(cell_views)
}

fn from_ascii_candidates_grid(input: &str) -> Result<Vec<CellView>> {
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
        .map(TryInto::<CellView>::try_into)
        .collect::<Result<Vec<_>>>()
}

#[allow(dead_code)]
fn from_terminal_candidates_grid(input: &str) -> Result<Vec<CellView>> {
    let stripped_input_bytes = strip_ansi_escapes::strip(input.as_bytes())?;
    let stripped_input = String::from_utf8(stripped_input_bytes)?;

    dbg!(&stripped_input);

    // TODO: implement
    //  split into multi-line rows
    //  split rows into multi-line cells
    //  extract numbers from cells

    todo!()
}

#[cfg(test)]
pub(crate) mod tests {
    use anyhow::Context;

    use crate::base::SudokuBase;
    use crate::grid::serialization::GridFormat;
    use crate::grid::Grid;
    use crate::samples;

    use super::*;

    pub(crate) static INPUT_GIVENS_LINE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/res/parser/givens_line.txt"
    ));

    pub(crate) static INPUT_GIVENS_GRID: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/res/parser/givens_grid.txt"
    ));

    pub(crate) static INPUT_CANDIDATES: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/res/parser/candidates.txt"
    ));

    #[test]
    fn test_from_givens_line() -> Result<()> {
        let cells = from_givens_line(INPUT_GIVENS_LINE)?;

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
    fn test_from_givens_grid() -> Result<()> {
        let cells = from_givens_grid(INPUT_GIVENS_GRID);

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
    fn test_from_candidates_grid() -> Result<()> {
        use crate::cell::view::{c, v};

        let cells = from_ascii_candidates_grid(INPUT_CANDIDATES)?;

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

    #[test]
    fn test_parse_cells_roundtrip() {
        let grid_formats = vec![
            GridFormat::GivensLine,
            GridFormat::GivensGrid,
            GridFormat::BinaryCandidatesLine,
            // FIXME: handle formats in parse_cells
            // GridFormat::CandidatesGrid,
        ];

        fn assert_grid_roundtrip<Base: SudokuBase>(
            grid_format: GridFormat,
            grid_string: &str,
            grid: Grid<Base>,
        ) {
            let cell_views = parse_cells(grid_string)
                .with_context(|| {
                    format!(
                        "parse_cells to parse:\n\
                        {grid_string}"
                    )
                })
                .unwrap();

            let parsed_grid = cell_views
                .try_into()
                .with_context(|| {
                    format!(
                        "Failed to convert cell_views to grid:\n\
                        {grid_string}\n\
                        {grid_string:?}"
                    )
                })
                .unwrap();

            assert_eq!(
                grid, parsed_grid,
                "Failed to roundtrip format {grid_format:?}:\n\
                    Original:\n\
                    {grid}\n\
                    \n\
                    Serialized:\n\
                    {grid_string}\n\
                    Parsed:\n\
                    {parsed_grid}"
            );
        }

        for grid_format in grid_formats {
            for (grid_string, grid) in samples::base_2()
                .into_iter()
                .map(|grid| (grid_format.render(&grid), grid))
            {
                assert_grid_roundtrip(grid_format, &grid_string, grid);
            }

            for (grid_string, grid) in samples::base_3()
                .into_iter()
                .map(|grid| (grid_format.render(&grid), grid))
            {
                assert_grid_roundtrip(grid_format, &grid_string, grid);
            }
        }
    }
}
