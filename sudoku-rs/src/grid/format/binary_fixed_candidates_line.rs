use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::grid::format::GridFormat;
use crate::grid::Grid;

/// New compact candidates grid format defined by [sudokuwiki.org](https://www.sudokuwiki.org/Sudoku_String_Definitions)
///
/// Used by the solver via the search parameter `bd`.
///
/// Differences from `BinaryCandidatesLine`:
/// - additional bit in the candidates bitset indicating a clue (fixed value)
/// - candidates bitset is encoded in base32 and padded with leading zeros to a fixed length.
///   In base 3, the length is 2 characters.
/// - no delimiters
///
/// # Example
/// `8104jk4s5e0ujalgnqhm0m0921d68mp2tgli3i413g8og18q059g3qiu11ikocac41q2okimieaei4oc0h1141o4i6mkakmk03a4q4r009jk5s0s03ks4cgsh821816s2s81116cisg803kc7cg174cceeae0h545c`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BinaryFixedCandidatesLine;

impl GridFormat for BinaryFixedCandidatesLine {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        use crate::cell::CellState;
        use radix_fmt::radix_32;

        grid.cells
            .iter()
            .map(|cell| {
                let mut bits: u32 = match cell.state() {
                    CellState::Value(value) | CellState::FixedValue(value) => {
                        2u32.pow(u32::from(value.into_u8() - 1))
                    }
                    CellState::Candidates(candidates) => candidates.integral().into(),
                };
                // Make space for fixed value bit
                bits <<= 1;
                if cell.has_fixed_value() {
                    bits += 1;
                }
                let base32string = format!("{}", radix_32(bits));
                let padded = format!("{base32string:0>2}");
                dbg!(padded)
            })
            .collect()
    }

    fn parse(self, _input: &str) -> crate::error::Result<Vec<DynamicCell>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    //noinspection SpellCheckingInspection
    #[test]
    fn test_render_binary_fixed_candidates_line() {
        let mut grid = samples::base_3().pop().unwrap();
        grid.set_all_direct_candidates();
        grid.get_mut((0, 1).try_into().unwrap())
            .set_value(2.try_into().unwrap());

        assert_eq!(
            BinaryFixedCandidatesLine.render(&grid),
            "8104jk4s5e0ujalgnqhm0m0921d68mp2tgli3i413g8og18q059g3qiu11ikocac41q2okimieaei4oc0h1141o4i6mkakmk03a4q4r009jk5s0s03ks4cgsh821816s2s81116cisg803kc7cg174cceeae0h545c"
        );
    }
}
