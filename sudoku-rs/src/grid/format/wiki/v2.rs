// FIXME: sudokuwiki's format has changed again:
//  https://www.sudokuwiki.org/Sudoku_String_Definitions
//  https://blueant1.github.io/puzzle-coding/documentation/puzzlecoding/encodingformats/
//  => Implement as a new format

use crate::base::BaseEnum;
use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::CellState;
use crate::cell::{Candidates, Cell};
use crate::error::Error;
use crate::error::Result;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormat;
use crate::grid::format::GridFormatCapabilities;
use crate::grid::format::GridFormatDetectAndParseCapability;
use crate::grid::format::GridFormatPreservesCellCandidates;
use crate::grid::format::GridFormatPreservesCellValue;
use crate::grid::Grid;
use crate::match_base_enum;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use std::fmt::Display;
use std::fmt::Write;

const HEADER_LENGTH: usize = 3;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BinaryCandidatesLineV2;

impl GridFormat for BinaryCandidatesLineV2 {
    fn capabilities(self) -> GridFormatCapabilities {
        GridFormatCapabilities {
            preserves_cell_value: GridFormatPreservesCellValue::ValueAndFixedState,
            preserves_cell_candidates: GridFormatPreservesCellCandidates::All,
            detect_and_parse: GridFormatDetectAndParseCapability::Detectable,
        }
    }
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String {
        use radix_fmt::radix_36;

        let payload = grid.all_cells().fold(String::new(), |mut output, cell| {
            let integer = cell_to_integer(cell);

            let serialized_integer = format!("{}", radix_36(integer));
            let width = cell_char_count(Base::ENUM);
            let _ = write!(output, "{serialized_integer:0>width$}");
            output
        });

        HeaderWithPayload {
            base: Base::ENUM,
            payload: &payload,
        }
        .to_string()
    }

    fn parse(self, input: &str) -> Result<DynamicGrid> {
        fn parse_base<Base: SudokuBase>(input: &str) -> Result<Vec<DynamicCell>> {
            input
                .as_bytes()
                .chunks(cell_char_count(Base::ENUM))
                .map(|cell_bytes_chunk| -> Result<DynamicCell> {
                    let integer = u32::from_str_radix(std::str::from_utf8(cell_bytes_chunk)?, 36)?;

                    Ok(integer_to_cell::<Base>(integer)?.into())
                })
                .collect()
        }
        let header: HeaderWithPayload = input.try_into()?;

        let dynamic_cells = match_base_enum!(header.base, parse_base::<Base>(header.payload)?);

        dynamic_cells.try_into()
    }
}

// Format header: https://blueant1.github.io/puzzle-coding/documentation/puzzlecoding/encodingformats#Header
#[derive(Debug)]
struct HeaderWithPayload<'a> {
    base: BaseEnum,
    payload: &'a str,
}

impl<'a> TryFrom<&'a str> for HeaderWithPayload<'a> {
    type Error = Error;

    fn try_from(input: &'a str) -> Result<Self> {
        ensure!(input.len() >= HEADER_LENGTH, "Input too short for header");
        let (header, payload) = input
            .split_at_checked(HEADER_LENGTH)
            .with_context(|| anyhow!("Failed to split input at header length"))?;
        let chars_vec: Vec<_> = header.chars().take(3).collect();
        let [puzzle_type_code, puzzle_size_code, encoding_version]: [char; HEADER_LENGTH] =
            chars_vec.try_into().map_err(|chars_vec: Vec<char>| {
                anyhow!(
                    "Header length must be 3 characters, instead got {}",
                    chars_vec.len()
                )
            })?;
        ensure!(
            puzzle_type_code == 'S',
            "Unexpected puzzle type code: {puzzle_type_code}"
        );
        ensure!(
            encoding_version == 'B',
            "Unexpected encoding version: {encoding_version}"
        );

        Ok(Self {
            base: match puzzle_size_code {
                '4' => BaseEnum::Base2,
                '9' => BaseEnum::Base3,
                'G' => BaseEnum::Base4,
                'P' => BaseEnum::Base5,
                unexpected_size_code => {
                    bail!("Unexpected puzzle size code: {unexpected_size_code}")
                }
            },
            payload,
        })
    }
}

impl Display for HeaderWithPayload<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { base, payload } = self;
        let size_code = match base {
            BaseEnum::Base2 => '4',
            BaseEnum::Base3 => '9',
            BaseEnum::Base4 => 'G',
            BaseEnum::Base5 => 'P',
        };
        write!(f, "S{size_code}B{payload}")
    }
}

const fn cell_char_count(base: BaseEnum) -> usize {
    match base {
        BaseEnum::Base2 => 1,
        BaseEnum::Base3 => 2,
        BaseEnum::Base4 => 4,
        BaseEnum::Base5 => 5,
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Offsets {
    solution: u32,
    candidates: u32,
    max_value: u32,
}

impl Offsets {
    const fn new<Base: SudokuBase>() -> Self {
        let solution_offset = Base::MAX_VALUE as u32;
        let candidates_offset = Base::MAX_VALUE as u32 * 2;
        let max_value = 2u32.pow(Base::MAX_VALUE as u32) - 1 + candidates_offset;

        Offsets {
            solution: solution_offset,
            candidates: candidates_offset,
            max_value,
        }
    }
}

// https://blueant1.github.io/puzzle-coding/documentation/puzzlecoding/cellcontenttransform#Encode
fn cell_to_integer<Base: SudokuBase>(cell: &Cell<Base>) -> u32 {
    let offsets = const { Offsets::new::<Base>() };

    let integer = match cell.state() {
        CellState::FixedValue(value) => u32::from(value.get()),
        CellState::Value(value) => u32::from(value.get()) + offsets.solution,
        CellState::Candidates(candidates) => {
            if candidates.is_empty() {
                0
            } else {
                Into::<u32>::into(candidates.integral()) + offsets.candidates
            }
        }
    };
    debug_assert!(
        integer <= offsets.max_value,
        "Cell integer value {integer} exceeds maximum value {}",
        offsets.max_value
    );
    integer
}

// https://blueant1.github.io/puzzle-coding/documentation/puzzlecoding/cellcontenttransform#Decode
fn integer_to_cell<Base: SudokuBase>(input: u32) -> Result<Cell<Base>> {
    let offsets = const { Offsets::new::<Base>() };

    ensure!(
        input <= offsets.max_value,
        "Cell integer value {input} exceeds maximum value {}",
        offsets.max_value
    );
    Ok(if input > offsets.candidates {
        Cell::with_candidates(Candidates::with_integral(
            Base::CandidatesIntegral::try_from(input - offsets.candidates)?,
        )?)
    } else if input > offsets.solution {
        Cell::with_value(u8::try_from(input - offsets.solution)?.try_into()?, false)
    } else if input > 0 {
        Cell::with_value(u8::try_from(input)?.try_into()?, true)
    } else {
        Cell::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::base::BaseEnum;
    use crate::cell::Value;
    use crate::grid::format::test_util::assert_parsed_grid;
    use crate::{
        base::SudokuBase,
        test_util::{test_all_bases, test_max_base4},
    };

    mod cell_char_count {
        use super::*;
        use radix_fmt::radix_36;

        test_all_bases!({
            // https://blueant1.github.io/puzzle-coding/documentation/puzzlecoding/fieldcoding
            let max_integer = Offsets::new::<Base>().max_value;
            let serialized = format!("{}", radix_36(max_integer));
            let expected_char_count = cell_char_count(Base::ENUM);
            assert_eq!(serialized.chars().count(), expected_char_count,);
        });
    }

    mod offsets {
        use super::*;

        // https://blueant1.github.io/puzzle-coding/documentation/puzzlecoding/cellcontenttransform#Details
        #[test]
        fn test_base_2() {
            assert_eq!(
                Offsets::new::<Base2>(),
                Offsets {
                    solution: 4,
                    candidates: 8,
                    max_value: 23,
                }
            );
        }

        #[test]
        fn test_base_3() {
            assert_eq!(
                Offsets::new::<Base3>(),
                Offsets {
                    solution: 9,
                    candidates: 18,
                    max_value: 529,
                }
            );
        }
        #[test]
        fn test_base_4() {
            assert_eq!(
                Offsets::new::<Base4>(),
                Offsets {
                    solution: 16,
                    candidates: 32,
                    max_value: 65_567,
                }
            );
        }
        #[test]
        fn test_base_5() {
            assert_eq!(
                Offsets::new::<Base5>(),
                Offsets {
                    solution: 25,
                    candidates: 50,
                    max_value: 33_554_481,
                }
            );
        }
    }

    mod cell {
        use super::*;

        mod integer_to_cell {
            use super::*;

            test_max_base4!({
                use itertools::Itertools;

                assert!((0..=Offsets::new::<Base>().max_value)
                    .map(|integer| { integer_to_cell::<Base>(integer).unwrap() })
                    .all_unique());

                integer_to_cell::<Base>(Offsets::new::<Base>().max_value + 1).unwrap_err();
            });
        }
        mod roundtrip {
            use super::*;

            fn assert_cell_roundtrip<Base: SudokuBase>(cell: &Cell<Base>) {
                let integer = cell_to_integer::<Base>(cell);
                let parsed_cell = integer_to_cell::<Base>(integer).unwrap();
                assert_eq!(&parsed_cell, cell);
            }

            test_all_bases!({
                for value in Value::<Base>::all() {
                    assert_cell_roundtrip(&Cell::with_value(value, false));
                    assert_cell_roundtrip(&Cell::with_value(value, true));
                }

                let candiates_iter: &mut dyn Iterator<Item = Candidates<Base>> = match Base::ENUM {
                    BaseEnum::Base2 | BaseEnum::Base3 | BaseEnum::Base4 => {
                        &mut Candidates::<Base>::iter_all_lexicographical()
                    }
                    BaseEnum::Base5 => &mut [
                        Candidates::new(),
                        Candidates::with_single(Value::default()),
                        Candidates::with_single(Value::max()),
                        Candidates::with_range(Value::default()..Value::middle()),
                        Candidates::with_range(Value::middle()..=Value::max()),
                        Candidates::all(),
                    ]
                    .into_iter(),
                };

                for candidates in candiates_iter {
                    assert_cell_roundtrip(&Cell::with_candidates(candidates));
                }
            });
        }
    }

    // Source: "Alternatively, with candidates (Version B)" https://www.sudokuwiki.org/Test_Strings
    static RENDERED_TEST_GRID: &str = "S9B0702160a0906164i038u1f7z025u05aj5u8r9208870m2a04ar028z4m2q6g7m655u890685010i06050k03080d074m046g8i6t5u89128522032y081u0a3w0924cq1vbv071u02261222023m2q04030i3m0108";

    fn get_test_grid() -> Grid<Base3> {
        use crate::cell::dynamic::{c, f, v};

        Grid::<Base3>::try_from(vec![
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
                c(vec![3]),
                c(vec![7]),
                f(4),
                c(vec![1, 5, 6, 7, 9]),
                f(2),
                c(vec![1, 5, 6, 9]),
            ],
            vec![
                c(vec![3, 5, 8]),
                c(vec![5, 7]),
                c(vec![2, 3, 5, 7, 8]),
                c(vec![9]),
                c(vec![1, 2, 4, 7, 8]),
                c(vec![7, 8]),
                c(vec![1, 2, 3, 5, 9]),
                f(6),
                c(vec![1, 2, 5, 9]),
            ],
            vec![f(1), v(9), f(6), f(5), c(vec![2]), f(3), f(8), v(4), f(7)],
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
        .unwrap()
    }

    #[test]
    fn test_render() {
        let test_grid = get_test_grid();
        let rendered_grid = BinaryCandidatesLineV2.render(&test_grid);
        assert_eq!(rendered_grid, RENDERED_TEST_GRID);
    }

    #[test]
    fn test_parse() {
        let parsed_grid = BinaryCandidatesLineV2.parse(RENDERED_TEST_GRID).unwrap();
        let expected_grid = get_test_grid();
        assert_parsed_grid(&expected_grid, &parsed_grid).unwrap();
    }
}
