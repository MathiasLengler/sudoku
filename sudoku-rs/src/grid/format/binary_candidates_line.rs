use crate::{
    base::SudokuBase,
    error::Result,
    grid::{
        dynamic::DynamicGrid,
        format::{
            GridFormat, GridFormatCapabilities, GridFormatPreservesCellCandidates,
            GridFormatPreservesCellValue,
        },
        Grid,
    },
};

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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BinaryCandidatesLine;

impl GridFormat for BinaryCandidatesLine {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueOnly,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::OnlyMultiple,
        }
    }

    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        use crate::cell::CellState;
        use itertools::Itertools;

        grid.all_cells()
            .map(|cell| match cell.state() {
                CellState::Value(value) | CellState::FixedValue(value) => {
                    2usize.pow(u32::from(value.get() - 1)).to_string()
                }
                CellState::Candidates(candidates) => candidates.integral().to_string(),
            })
            .join(",")
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        let mut cell_views = vec![];

        for cell_str in input.split(',') {
            let bits = cell_str.parse::<u32>()?;
            cell_views.push(bits.try_into()?);
        }

        cell_views.try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    #[test]
    fn test_render_binary_candidates_line() {
        let mut grid = samples::base_3().into_iter().next().unwrap();
        grid.set_all_direct_candidates();

        assert_eq!(
            BinaryCandidatesLine.render(&grid),
            "128,43,314,78,87,15,309,344,381,283,11,4,32,211,139,401,472,345,57,64,56,140,256,141,2,152,61,303,16,298,390,166,64,417,394,299,295,167,290,390,8,16,64,386,291,362,170,362,1,162,418,432,4,314,94,14,1,334,70,270,276,32,128,110,46,128,16,102,302,260,1,326,118,256,114,198,231,167,8,82,86"
        );
    }
}
