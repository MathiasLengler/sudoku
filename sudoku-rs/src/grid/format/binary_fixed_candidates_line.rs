use anyhow::bail;

use crate::base::consts::*;
use crate::base::SudokuBase;
use crate::cell::candidates_cell::CandidatesCell;
use crate::cell::dynamic::DynamicCell;
use crate::cell::{Candidates, Cell};
use crate::error::Result;
use crate::grid::format::GridFormat;
use crate::grid::Grid;

/// New compact candidates grid format defined by [sudokuwiki.org](https://www.sudokuwiki.org/Sudoku_String_Definitions)
///
/// Used by the solver via the search parameter `bd`.
///
/// Differences from `BinaryCandidatesLine`:
/// - additional bit in the candidates bitset indicating a clue (fixed value)
/// - candidates bitset is encoded in base32 and padded with leading zeros to a fixed length:
///   - Base 2: 1
///   - Base 3: 2
///   - Base 4: 4
///   - Base 5: 6
/// - no delimiters
///
/// # Example
/// `8104jk4s5e0ujalgnqhm0m0921d68mp2tgli3i413g8og18q059g3qiu11ikocac41q2okimieaei4oc0h1141o4i6mkakmk03a4q4r009jk5s0s03ks4cgsh821816s2s81116cisg803kc7cg174cceeae0h545c`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BinaryFixedCandidatesLine;

impl GridFormat for BinaryFixedCandidatesLine {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        use radix_fmt::radix_32;

        grid.all_cells()
            .map(|cell| {
                let candidates_cell: CandidatesCell<_> = cell.clone().into();

                let mut bits: u32 = candidates_cell.candidates.integral().into();
                // Make space for fixed value bit
                bits <<= 1;
                if cell.has_fixed_value() {
                    bits += 1;
                }
                let base32string = format!("{}", radix_32(bits));
                let width = Base::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS;
                format!("{base32string:0>width$}")
            })
            .collect()
    }

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>> {
        fn parse_base<Base: SudokuBase>(input: &str) -> Result<Vec<DynamicCell>> {
            input
                .as_bytes()
                .chunks(Base::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS)
                .map(|cell_bytes_chunk| -> Result<DynamicCell> {
                    let mut bits = u32::from_str_radix(std::str::from_utf8(cell_bytes_chunk)?, 32)?;

                    let is_fixed_value = (bits & 1) == 1;

                    bits >>= 1; // Shift the bits to remove the flag

                    let candidates_cell =
                        CandidatesCell::<Base>::with_candidates(Candidates::with_integral(
                            Base::CandidatesIntegral::try_from(bits).unwrap(),
                        ));

                    Ok(if let Some(value) = candidates_cell.value() {
                        Cell::with_value(value, is_fixed_value).into()
                    } else {
                        Cell::with_candidates(candidates_cell.candidates).into()
                    })
                })
                .collect()
        }

        const BASE_2_CHAR_COUNT: usize =
            (Base2::CELL_COUNT as usize) * Base2::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS;
        const BASE_3_CHAR_COUNT: usize =
            (Base3::CELL_COUNT as usize) * Base3::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS;
        const BASE_4_CHAR_COUNT: usize =
            (Base4::CELL_COUNT as usize) * Base4::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS;
        const BASE_5_CHAR_COUNT: usize =
            (Base5::CELL_COUNT as usize) * Base5::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS;

        match input.chars().count() {
            BASE_2_CHAR_COUNT => parse_base::<Base2>(input),
            BASE_3_CHAR_COUNT => parse_base::<Base3>(input),
            BASE_4_CHAR_COUNT => parse_base::<Base4>(input),
            BASE_5_CHAR_COUNT => parse_base::<Base5>(input),
            unexpected_char_count => bail!("Unexpected char count: {unexpected_char_count}"),
        }
    }

    fn do_fix_all_values(self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::format::test_util::assert_grid_format_roundtrip;
    use crate::samples;

    use super::*;

    //noinspection SpellCheckingInspection
    #[test]
    fn test_render() {
        let mut grid = samples::base_3().pop().unwrap();
        grid.set_all_direct_candidates();
        grid.get_mut((0, 1).try_into().unwrap())
            .set_value(2.try_into().unwrap());

        assert_eq!(
            BinaryFixedCandidatesLine.render(&grid),
            "8104jk4s5e0ujalgnqhm0m0921d68mp2tgli3i413g8og18q059g3qiu11ikocac41q2okimieaei4oc0h1141o4i6mkakmk03a4q4r009jk5s0s03ks4cgsh821816s2s81116cisg803kc7cg174cceeae0h545c"
        );
    }

    #[test]
    fn test_parse() {
        for grid in samples::base_2() {
            assert_grid_format_roundtrip(&grid, BinaryFixedCandidatesLine).unwrap();
        }

        for grid in samples::base_3() {
            assert_grid_format_roundtrip(&grid, BinaryFixedCandidatesLine).unwrap();
        }

        let mut grid = Grid::<Base4>::new();
        grid.set_all_direct_candidates();
        assert_grid_format_roundtrip(&grid, BinaryFixedCandidatesLine).unwrap();

        let mut grid = Grid::<Base5>::new();
        grid.set_all_direct_candidates();
        assert_grid_format_roundtrip(&grid, BinaryFixedCandidatesLine).unwrap();
    }
}
