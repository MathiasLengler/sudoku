use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use anyhow::{bail, ensure, format_err};
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use binary_candidates_line::*;
pub use binary_fixed_candidates_line::*;
pub use candidates_grid::*;
pub use candidates_grid_compact::*;
pub use values_grid::*;
pub use values_line::ValuesLine;

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;

mod binary_candidates_line;
mod binary_fixed_candidates_line;
mod candidates_grid;
mod candidates_grid_compact;
mod values_grid;
mod values_line;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GridFormatPreservesCellValue {
    /// Perserves the value, but not the fixed (clue) state.
    ValueOnly,
    /// Perserves both value and fixed (clue) state.
    ValueAndFixedState,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GridFormatPreservesCellCandidates {
    /// Perserves no candidates, e.g. an empty cell.
    /// Any candidates will be lost.
    Empty,
    /// Preserves empty and multiple candidates.
    /// Single candidates may be converted to values.
    OnlyMultiple,
    /// Preserves empty, single and multiple candidates.
    All,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GridFormatCapabilities {
    pub preserves_cell_value: GridFormatPreservesCellValue,
    pub preserves_cell_candidates: GridFormatPreservesCellCandidates,
}

#[enum_dispatch(GridFormatEnum)]
pub trait GridFormat: Debug + Copy + Clone + Eq + Sized {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String;

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>>;

    fn capabilities(self) -> GridFormatCapabilities;

    fn parse_and_validate_cell_count(self, input: &str) -> Result<Vec<DynamicCell>> {
        use crate::base::consts::ALL_CELL_COUNTS;

        let mut dynamic_cells = GridFormat::parse(self, input)?;

        let actual_cell_count = dynamic_cells.len().try_into()?;

        ensure!(
            ALL_CELL_COUNTS.contains(&actual_cell_count),
            "Unexpected cell count {actual_cell_count}, expected one of: {ALL_CELL_COUNTS:?}"
        );

        if self.capabilities().preserves_cell_value == GridFormatPreservesCellValue::ValueOnly {
            // If the format does not preserve fixed state, assume all values are fixed.
            for dynamic_cell in &mut dynamic_cells {
                if let DynamicCell::Value { fixed, value } = dynamic_cell {
                    if value.0 != 0 {
                        *fixed = true;
                    }
                }
            }
        }

        Ok(dynamic_cells)
    }

    fn name(self) -> String {
        format!("{self:?}")
    }
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[enum_dispatch]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GridFormatEnum {
    BinaryCandidatesLine,
    BinaryFixedCandidatesLine,
    CandidatesGridANSIStyled,
    CandidatesGridPlain,
    CandidatesGridCompact,
    ValuesLine,
    ValuesGrid,
    // TODO: dymaic grid serde JSON
}

impl GridFormatEnum {
    pub fn all() -> Vec<Self> {
        vec![
            BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            CandidatesGridANSIStyled.into(),
            CandidatesGridPlain.into(),
            CandidatesGridCompact.into(),
            ValuesLine.into(),
            ValuesGrid.into(),
        ]
    }

    pub fn detect_and_parse(input: &str) -> Result<Vec<DynamicCell>> {
        let input = input.trim();
        if input.is_empty() {
            bail!("Unexpected empty input")
        }

        let cell_views = if input.contains('\n') {
            CandidatesGridANSIStyled
                .parse_and_validate_cell_count(input)
                .or_else(|_| CandidatesGridCompact.parse_and_validate_cell_count(input))
                .or_else(|_| ValuesGrid.parse_and_validate_cell_count(input))?
        } else {
            ValuesLine
                .parse_and_validate_cell_count(input)
                .or_else(|_| BinaryFixedCandidatesLine.parse_and_validate_cell_count(input))
                .or_else(|_| BinaryCandidatesLine.parse_and_validate_cell_count(input))?
        };

        Ok(cell_views)
    }
}

impl Serialize for GridFormatEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (variant_index, variant) = match *self {
            Self::BinaryCandidatesLine(_) => (0, "BinaryCandidatesLine"),
            Self::BinaryFixedCandidatesLine(_) => (1, "BinaryFixedCandidatesLine"),
            Self::CandidatesGridANSIStyled(_) => (2, "CandidatesGridANSIStyled"),
            Self::CandidatesGridPlain(_) => (3, "CandidatesGridPlain"),
            Self::CandidatesGridCompact(_) => (4, "CandidatesGridCompact"),
            Self::ValuesLine(_) => (5, "ValuesLine"),
            Self::ValuesGrid(_) => (6, "ValuesGrid"),
        };

        serializer.serialize_unit_variant("Strategy", variant_index, variant)
    }
}

impl<'de> Deserialize<'de> for GridFormatEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GridFormatVisitor;

        impl Visitor<'_> for GridFormatVisitor {
            type Value = GridFormatEnum;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a valid grid format name")
            }

            fn visit_str<E>(self, strategy_name: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                strategy_name.parse().map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(GridFormatVisitor)
    }
}

impl FromStr for GridFormatEnum {
    type Err = Error;

    fn from_str(grid_format_name: &str) -> Result<Self> {
        GridFormatEnum::all()
            .into_iter()
            .find(|grid_format| grid_format.name() == grid_format_name)
            .ok_or_else(|| format_err!("Unexpected grid format name: {grid_format_name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grid::format::test_util::assert_grid_format_roundtrip_unchanged_detect, samples};
    use anyhow::Context;

    mod grid_format_enum {
        use super::*;
        #[test]
        fn test_serde_round_trip() {
            let all_strategies = GridFormatEnum::all();

            let json_string = serde_json::to_string(&all_strategies).unwrap();

            let all_strategies_round_tripped: Vec<GridFormatEnum> =
                serde_json::from_str(&json_string).unwrap();

            assert_eq!(all_strategies, all_strategies_round_tripped);
        }
    }

    #[test]
    fn test_detect_and_parse_cells_roundtrip() {
        // TODO: move format capabilities to the format itself
        //  declare unit test for each format
        //  define helper which tests based on the declared capabilities

        // Grid formats which preserve:
        // - cell value
        let grid_formats: Vec<GridFormatEnum> = vec![
            BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            CandidatesGridANSIStyled.into(),
            CandidatesGridPlain.into(),
            CandidatesGridCompact.into(),
            ValuesLine.into(),
            ValuesGrid.into(),
        ];

        // TODO: refactor other duplicated base handling code
        macro_rules! for_test_grids {
            (|$grid:ident| $block:block) => {
                #[allow(unused_mut)]
                for mut $grid in samples::base_2() $block
                #[allow(unused_mut)]
                for mut $grid in samples::base_3() $block
                #[allow(unused_mut)]
                for mut $grid in samples::base_4() $block
                #[allow(unused_mut)]
                for mut $grid in samples::base_5() $block
            };
        }

        for grid_format in grid_formats {
            for_test_grids!(|grid| {
                grid.fix_all_values();

                assert_grid_format_roundtrip_unchanged_detect(grid_format, &grid)
                    .with_context(|| "Test cell value roundtrip".to_string())
                    .unwrap();
            });
        }

        // Grid formats which preserve:
        // - cell value
        // - cell value fixed state
        let grid_formats: Vec<GridFormatEnum> = vec![
            // BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            // CandidatesGridANSIStyled.into(),
            // CandidatesGridPlain.into(),
            // CandidatesGridCompact.into(),
            // ValuesLine.into(),
            // ValuesGrid.into(),
        ];

        for grid_format in grid_formats {
            for_test_grids!(|grid| {
                grid.fix_all_values();
                assert_grid_format_roundtrip_unchanged_detect(grid_format, &grid)
                    .with_context(|| "Test cell fixed value roundtrip".to_string())
                    .unwrap();
                grid.unfix_all_values();
                assert_grid_format_roundtrip_unchanged_detect(grid_format, &grid)
                    .with_context(|| "Test cell unfixed value roundtrip".to_string())
                    .unwrap();
            });
        }

        // Grid formats which preserve:
        // - cell value
        // - cell candidates
        let grid_formats: Vec<GridFormatEnum> = vec![
            // BinaryCandidatesLine.into(),
            // BinaryFixedCandidatesLine.into(),
            CandidatesGridANSIStyled.into(),
            CandidatesGridPlain.into(),
            CandidatesGridCompact.into(),
            // ValuesLine.into(),
            // ValuesGrid.into(),
        ];
        for grid_format in grid_formats {
            for_test_grids!(|grid| {
                assert_grid_format_roundtrip_unchanged_detect(grid_format, &grid)
                    .with_context(|| "Test cell candidates roundtrip".to_string())
                    .unwrap();
            });
        }

        // Grid formats which preserve:
        // - cell value
        // - cell value fixed state
        // - cell multiple candidates
        let grid_formats: Vec<GridFormatEnum> = vec![
            // BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            // CandidatesGridANSIStyled.into(),
            // CandidatesGridPlain.into(),
            // CandidatesGridCompact.into(),
            // ValuesLine.into(),
            // ValuesGrid.into(),
        ];
        for grid_format in grid_formats {
            for_test_grids!(|grid| {
                // Delete single candidates which would get converted to a value.
                grid.all_candidates_positions().into_iter().for_each(|pos| {
                    if grid.get(pos).candidates().unwrap().to_single().is_some() {
                        grid.get_mut(pos).delete();
                    }
                });
                // Unfix each second value.
                grid.all_value_positions()
                    .into_iter()
                    .enumerate()
                    .filter(|&(i, _)| i % 2 == 0)
                    .for_each(|(_, pos)| grid.get_mut(pos).unfix());
                assert_grid_format_roundtrip_unchanged_detect(grid_format, &grid)
                    .with_context(|| "Test cell multiple candidates roundtrip".to_string())
                    .unwrap();
            });
        }
    }

    mod via_capabilities {
        use super::*;
        use crate::{
            cell::{Candidates, Cell, Value},
            grid::format::test_util::{
                assert_grid_format_roundtrip, assert_grid_format_roundtrip_unchanged,
            },
        };

        // TODO: test with detect_parse_format = true

        mod cell_candidates {
            use super::*;
            use crate::test_util::test_all_bases;

            fn assert_preserves_cell_candidates<Base: SudokuBase, F: GridFormat>(
                grid_format: F,
                grid_with_candidates: &Grid<Base>,
            ) {
                assert!(
                    grid_with_candidates.all_value_positions().is_empty(),
                    "Not all values are empty in grid:\n{grid_with_candidates}"
                );

                let capabilities = grid_format.capabilities();
                match capabilities.preserves_cell_candidates {
                    GridFormatPreservesCellCandidates::Empty => {
                        let empty_grid = Grid::<Base>::new();
                        assert_grid_format_roundtrip(
                            grid_format,
                            false,
                            grid_with_candidates,
                            &empty_grid,
                        )
                        .unwrap();
                    }
                    GridFormatPreservesCellCandidates::OnlyMultiple => {
                        let grid_without_single_candidates = {
                            let mut grid = grid_with_candidates.clone();
                            for pos in grid.all_candidates_positions() {
                                if grid.get(pos).candidates().unwrap().is_single() {
                                    grid.get_mut(pos).delete();
                                }
                            }
                            grid
                        };

                        assert_grid_format_roundtrip_unchanged(
                            grid_format,
                            &grid_without_single_candidates,
                        )
                        .unwrap();
                    }
                    GridFormatPreservesCellCandidates::All => {
                        assert_grid_format_roundtrip_unchanged(grid_format, grid_with_candidates)
                            .unwrap();
                    }
                }
            }

            mod lexicographical_filled {
                use super::*;

                test_all_bases!({
                    for grid_format in GridFormatEnum::all() {
                        let grid = Grid::<Base>::with(
                            Candidates::iter_all_lexicographical()
                                .take(Base::CELL_COUNT.into())
                                .map(Cell::with_candidates)
                                .collect(),
                        )
                        .unwrap();
                        assert_preserves_cell_candidates(grid_format, &grid);
                    }
                });
            }
        }

        mod cell_value {
            use super::*;
            use crate::test_util::test_all_bases;

            fn assert_preserves_cell_value<Base: SudokuBase, F: GridFormat>(
                grid_format: F,
                grid_with_fixed_values: &Grid<Base>,
            ) {
                assert!(
                    grid_with_fixed_values
                        .all_unfixed_value_positions()
                        .is_empty(),
                    "Not all values are fixed in grid:\n{grid_with_fixed_values}"
                );
                assert!(
                    grid_with_fixed_values.are_all_candidates_empty(),
                    "Not all candidates are empty in grid:\n{grid_with_fixed_values}"
                );
                let grid_with_unfixed_values = {
                    let mut grid = grid_with_fixed_values.clone();
                    grid.unfix_all_values();
                    grid
                };

                // Fixed values remain fixed.
                assert_grid_format_roundtrip_unchanged(grid_format, grid_with_fixed_values)
                    .unwrap();

                match grid_format.capabilities().preserves_cell_value {
                    GridFormatPreservesCellValue::ValueOnly => {
                        // Unfixed values get converted to fixed values.
                        assert_grid_format_roundtrip(
                            grid_format,
                            false,
                            &grid_with_unfixed_values,
                            grid_with_fixed_values,
                        )
                        .unwrap();
                    }
                    GridFormatPreservesCellValue::ValueAndFixedState => {
                        // Unfixed values remain unfixed.
                        assert_grid_format_roundtrip_unchanged(
                            grid_format,
                            &grid_with_unfixed_values,
                        )
                        .unwrap();
                    }
                }
            }

            // TODO: test via all sample grids

            mod filled {
                use super::*;

                test_all_bases!({
                    for grid_format in GridFormatEnum::all() {
                        for value in [Value::default(), Value::middle(), Value::max()] {
                            let grid_with_fixed_values =
                                Grid::<Base>::filled_with(Cell::with_value(value, true));
                            assert_preserves_cell_value(grid_format, &grid_with_fixed_values);
                        }
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod test_util {
    use super::*;
    use anyhow::Context;

    pub(crate) fn assert_grid_equals_dynamic_cells<Base: SudokuBase>(
        expected_grid: &Grid<Base>,
        dynamic_cells: &[DynamicCell],
    ) -> Result<()> {
        let parsed_grid: Grid<Base> = dynamic_cells
            .to_vec()
            .try_into()
            .with_context(|| format!("Failed to convert cells to grid:\n{dynamic_cells:#?}"))?;

        ensure!(
            expected_grid == &parsed_grid,
            "Mismatched grids; expected:\n{expected_grid}\nParsed:\n{parsed_grid}"
        );

        Ok(())
    }

    pub(crate) fn assert_grid_format_roundtrip<Base: SudokuBase, F: GridFormat>(
        grid_format: F,
        detect_parse_format: bool,
        grid_to_render: &Grid<Base>,
        expected_parsed_grid: &Grid<Base>,
    ) -> Result<()> {
        (|| {
            let grid_string = grid_format.render(grid_to_render);

            let parsed_cells = if detect_parse_format {
                GridFormatEnum::detect_and_parse(&grid_string).with_context(|| {
                    format!("Failed to detect and parse grid string:\n{grid_string}")
                })?
            } else {
                grid_format
                    .parse_and_validate_cell_count(&grid_string)
                    .with_context(|| format!("Failed to parse grid string:\n{grid_string}"))?
            };

            assert_grid_equals_dynamic_cells(expected_parsed_grid, &parsed_cells).with_context(
                || {
                    format!(
                        "Failed to compare parsed cells to expected parsed grid:\n{grid_string}"
                    )
                },
            )
        })()
        .with_context(|| {
            format!(
                "Failed to roundtrip format {} with grid:\n{grid_to_render}",
                grid_format.name()
            )
        })
    }

    pub(crate) fn assert_grid_format_roundtrip_unchanged<Base: SudokuBase, F: GridFormat>(
        grid_format: F,
        grid: &Grid<Base>,
    ) -> Result<()> {
        assert_grid_format_roundtrip(grid_format, false, grid, grid)
    }

    pub(crate) fn assert_grid_format_roundtrip_unchanged_detect<Base: SudokuBase>(
        grid_format: GridFormatEnum,
        grid: &Grid<Base>,
    ) -> Result<()> {
        assert_grid_format_roundtrip(grid_format, true, grid, grid)
    }
}
