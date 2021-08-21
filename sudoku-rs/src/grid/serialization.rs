use crate::base::SudokuBase;
use crate::grid::Grid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GridFormat {
    GivensLine,
    GivensGrid,
    Candidates,
}

impl GridFormat {
    pub fn render<Base: SudokuBase>(&self, grid: &Grid<Base>) -> String {
        match self {
            GridFormat::GivensLine => render_givens_line(grid),
            GridFormat::GivensGrid => render_givens_grid(grid),
            // TODO: show candidates (compare with exchange formats)
            GridFormat::Candidates => {
                unimplemented!()
            }
        }
    }
}

fn render_givens_line<Base: SudokuBase>(grid: &Grid<Base>) -> String {
    grid.cells.iter().map(ToString::to_string).collect()
}

fn render_givens_grid<Base: SudokuBase>(grid: &Grid<Base>) -> String {
    // TODO: implement using prettytable-rs
    use itertools::Itertools;
    use ndarray::Axis;

    const PADDING: usize = 3;

    let horizontal_block_separator =
        "-".repeat(Grid::<Base>::base_usize() + (PADDING * Grid::<Base>::side_length_usize()));

    Itertools::intersperse(
        grid.cells
            .rows()
            .into_iter()
            .map(|row| {
                row.axis_chunks_iter(Axis(0), Grid::<Base>::base_usize())
                    .map(|block_row| {
                        block_row
                            .iter()
                            .map(|cell| {
                                format!("{:>PADDING$}", cell.to_string(), PADDING = PADDING)
                            })
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .collect::<Vec<String>>()
            .chunks(Grid::<Base>::base_usize()),
        &[horizontal_block_separator],
    )
    .flatten()
    .cloned()
    .collect::<Vec<String>>()
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_givens_line() {
        use crate::samples::base_3;
        let grid = base_3().pop().unwrap();

        assert_eq!(
            "800000000003600000070090200050007000000045700000100030001000068008500010090000400",
            GridFormat::GivensLine.render(&grid)
        );
    }
    #[test]
    fn test_render_givens_grid() {
        use crate::samples::base_3;
        let grid = base_3().pop().unwrap();

        assert_eq!(
            "  8  0  0|  0  0  0|  0  0  0
  0  0  3|  6  0  0|  0  0  0
  0  7  0|  0  9  0|  2  0  0
------------------------------
  0  5  0|  0  0  7|  0  0  0
  0  0  0|  0  4  5|  7  0  0
  0  0  0|  1  0  0|  0  3  0
------------------------------
  0  0  1|  0  0  0|  0  6  8
  0  0  8|  5  0  0|  0  1  0
  0  9  0|  0  0  0|  4  0  0",
            GridFormat::GivensGrid.render(&grid)
        );
    }
}
