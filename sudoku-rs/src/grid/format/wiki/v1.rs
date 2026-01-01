use crate::base::consts::*;
use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::{Candidates, Cell};
use crate::error::Result;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormat;
use crate::grid::format::GridFormatCapabilities;
use crate::grid::format::GridFormatDetectAndParseCapability;
use crate::grid::format::GridFormatPreservesCellCandidates;
use crate::grid::format::GridFormatPreservesCellValue;
use crate::grid::Grid;
use anyhow::bail;
use std::fmt::Write;

/// Compact candidates grid format defined by [sudokuwiki.org](https://www.sudokuwiki.org/Sudoku_String_Definitions) as "Version 1".
///
/// Used by their solver via the search parameter `bd`.
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
/// # Note
/// This format does not differentiate between single candidates and values.
///
/// # Example
/// `8104jk4s5e0ujalgnqhm0m0921d68mp2tgli3i413g8og18q059g3qiu11ikocac41q2okimieaei4oc0h1141o4i6mkakmk03a4q4r009jk5s0s03ks4cgsh821816s2s81116cisg803kc7cg174cceeae0h545c`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BinaryCandidatesLineV1;

impl GridFormat for BinaryCandidatesLineV1 {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueAndFixedState,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::OnlyMultiple,
            // Is confused with `ValuesLine` in base 2 for a grid containing only candidates.
            detect_and_parse: GridFormatDetectAndParseCapability::Lossy,
        }
    }
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        use radix_fmt::radix_32;

        grid.all_cells().fold(String::new(), |mut output, cell| {
            let candidates: Candidates<_> = cell.clone().into();

            let mut bits: u32 = candidates.integral().into();
            // Make space for fixed value bit
            bits <<= 1;
            if cell.has_fixed_value() {
                bits += 1;
            }
            let base32string = format!("{}", radix_32(bits));
            let width = Base::ENUM.binary_fixed_candidates_line_cell_chars();
            let _ = write!(output, "{base32string:0>width$}");
            output
        })
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        fn parse_base<Base: SudokuBase>(input: &str) -> Result<Vec<DynamicCell>> {
            input
                .as_bytes()
                .chunks(Base::ENUM.binary_fixed_candidates_line_cell_chars())
                .map(|cell_bytes_chunk| -> Result<DynamicCell> {
                    let mut bits = u32::from_str_radix(std::str::from_utf8(cell_bytes_chunk)?, 32)?;

                    let is_fixed_value = (bits & 1) == 1;

                    bits >>= 1; // Shift the bits to remove the flag

                    let candidates: Candidates<Base> =
                        Candidates::with_integral(Base::CandidatesIntegral::try_from(bits)?)?;

                    Ok(DynamicCell::from(Cell::from((candidates, is_fixed_value))))
                })
                .collect()
        }

        const BASE_2_CHAR_COUNT: usize =
            (Base2::CELL_COUNT as usize) * Base2::ENUM.binary_fixed_candidates_line_cell_chars();
        const BASE_3_CHAR_COUNT: usize =
            (Base3::CELL_COUNT as usize) * Base3::ENUM.binary_fixed_candidates_line_cell_chars();
        const BASE_4_CHAR_COUNT: usize =
            (Base4::CELL_COUNT as usize) * Base4::ENUM.binary_fixed_candidates_line_cell_chars();
        const BASE_5_CHAR_COUNT: usize =
            (Base5::CELL_COUNT as usize) * Base5::ENUM.binary_fixed_candidates_line_cell_chars();

        let dynamic_cells = match input.chars().count() {
            BASE_2_CHAR_COUNT => parse_base::<Base2>(input),
            BASE_3_CHAR_COUNT => parse_base::<Base3>(input),
            BASE_4_CHAR_COUNT => parse_base::<Base4>(input),
            BASE_5_CHAR_COUNT => parse_base::<Base5>(input),
            unexpected_char_count => bail!("Unexpected char count: {unexpected_char_count}"),
        }?;
        dynamic_cells.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::format::test_util::assert_parsed_grid;
    use crate::position::Position;
    use crate::samples;

    #[test]
    fn test_render() {
        let mut grid = samples::base_3().into_iter().next().unwrap();
        grid.set_all_direct_candidates();
        grid.get_mut((0, 1).try_into().unwrap())
            .set_value(2.try_into().unwrap());

        assert_eq!(
            BinaryCandidatesLineV1.render(&grid),
            "8104jk4s5e0ujalgnqhm0m0921d68mp2tgli3i413g8og18q059g3qiu11ikocac41q2okimieaei4oc0h1141o4i6mkakmk03a4q4r009jk5s0s03ks4cgsh821816s2s81116cisg803kc7cg174cceeae0h545c"
        );
    }

    #[test]
    fn test_parse() {
        use crate::cell::dynamic::{c, f, v};

        let parsed_grid = BinaryCandidatesLineV1.parse(
            // Source: "Alternatively, with candidates (old style)" https://www.sudokuwiki.org/Test_Strings
            "41051g02g1211g9009io22gq05c011mic0iij881ha08400hn205j29850dcg0cmc0he21h603g021110409810g41980hdci0e6c0he18h63g095g8130027kg13krg32pi4130053o183g0570500h09g0700381"
        ).unwrap();
        let expected_grid = Grid::<Base3>::try_from(vec![
            vec![
                f(7),
                f(2),
                c(vec![4, 5]),
                v(1),
                f(9),
                f(6),
                c(vec![4, 5]),
                c(vec![5, 8]),
                f(3),
            ],
            vec![
                c(vec![3, 4, 6, 9]),
                c(vec![1, 6]),
                c(vec![1, 3, 4, 9]),
                f(2),
                c(vec![7, 8]),
                f(5),
                c(vec![1, 4, 6, 7, 9]),
                c(vec![7, 8]),
                c(vec![1, 4, 6, 9]),
            ],
            vec![
                c(vec![3, 5, 6, 9]),
                f(8),
                c(vec![1, 3, 5, 9]),
                v(3),
                v(7),
                f(4),
                c(vec![1, 5, 6, 7, 9]),
                f(2),
                c(vec![1, 5, 6, 9]),
            ],
            vec![
                c(vec![3, 5, 8]),
                c(vec![5, 7]),
                c(vec![2, 3, 5, 7, 8]),
                v(9),
                c(vec![1, 2, 4, 7, 8]),
                c(vec![7, 8]),
                c(vec![1, 2, 3, 5, 9]),
                f(6),
                c(vec![1, 2, 5, 9]),
            ],
            vec![f(1), v(9), f(6), f(5), v(2), f(3), f(8), v(4), f(7)],
            vec![
                c(vec![3, 5, 8]),
                f(4),
                c(vec![2, 3, 5, 7, 8]),
                c(vec![6, 9]),
                c(vec![1, 2, 6, 7, 8]),
                c(vec![7, 8]),
                c(vec![1, 2, 3, 5, 9]),
                c(vec![3, 5]),
                c(vec![1, 2, 5, 9]),
            ],
            vec![
                c(vec![4, 5, 6]),
                f(3),
                c(vec![4, 5, 7]),
                f(8),
                c(vec![5, 6]),
                v(1),
                c(vec![2, 4, 5, 6, 7]),
                f(9),
                c(vec![2, 4, 5, 6]),
            ],
            vec![
                c(vec![4, 5, 6, 8, 9]),
                c(vec![1, 5, 6]),
                c(vec![1, 4, 5, 8, 9]),
                f(7),
                c(vec![5, 6]),
                f(2),
                c(vec![3, 4, 5, 6]),
                c(vec![3, 5]),
                c(vec![4, 5, 6]),
            ],
            vec![
                f(2),
                c(vec![5, 6, 7]),
                c(vec![5, 7]),
                f(4),
                f(3),
                v(9),
                c(vec![5, 6, 7]),
                f(1),
                f(8),
            ],
        ])
        .unwrap();
        assert_parsed_grid(&expected_grid, &parsed_grid).unwrap();
    }

    #[test]
    fn test_single_candidate_to_unfixed_value() {
        type Base = Base2;
        let grid = {
            let mut grid = Grid::<Base>::new();
            grid.get_mut(Position::top_left())
                .set_candidates(Candidates::with_single(1.try_into().unwrap()));
            grid
        };

        let grid_roundtrip = Grid::<Base>::try_from(
            BinaryCandidatesLineV1
                .parse(&BinaryCandidatesLineV1.render(&grid))
                .unwrap(),
        )
        .unwrap();

        assert_eq!(
            grid_roundtrip[Position::top_left()],
            Cell::with_value(1.try_into().unwrap(), false)
        );
    }
}
