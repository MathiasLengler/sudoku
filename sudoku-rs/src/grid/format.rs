use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::grid::dynamic::DynamicGrid;
use anyhow::{bail, format_err};
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

pub use candidates_grid::{CandidatesGridANSIStyled, CandidatesGridPlain};
pub use candidates_grid_compact::CandidatesGridCompact;
pub use json::Json;
pub use values_grid::ValuesGrid;
pub use values_line::ValuesLine;
pub use wiki::v0::BinaryCandidatesLineV0;
pub use wiki::v1::BinaryCandidatesLineV1;
pub use wiki::v2::BinaryCandidatesLineV2;

mod candidates_grid;
mod candidates_grid_compact;
mod json;
mod values_grid;
mod values_line;
mod wiki;

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
pub enum GridFormatDetectAndParseCapability {
    /// Detects and parses on its own.
    Detectable,
    /// Is detected as another format, but roundtrips correctly.
    DetectableViaOtherFormat,
    /// The format cannot be detected and parsed without loss.
    Lossy,
    /// The format is not supported by `detect_and_parse`.
    Unsupported,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GridFormatCapabilities {
    pub preserves_cell_value: GridFormatPreservesCellValue,
    pub preserves_cell_candidates: GridFormatPreservesCellCandidates,
    pub detect_and_parse: GridFormatDetectAndParseCapability,
}

#[enum_dispatch(GridFormatEnum)]
pub trait GridFormat: Debug + Copy + Clone + Eq + Sized + Into<GridFormatEnum> {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String;

    fn parse(self, input: &str) -> Result<DynamicGrid>;

    fn capabilities(self) -> GridFormatCapabilities;

    fn parse_and_post_process(self, input: &str) -> Result<DynamicGrid> {
        let mut dynamic_grid = GridFormat::parse(self, input)?;

        if self.capabilities().preserves_cell_value == GridFormatPreservesCellValue::ValueOnly {
            // If the format does not preserve fixed state, assume all values are fixed.
            for dynamic_cell in &mut dynamic_grid {
                if let DynamicCell::Value { fixed, value } = dynamic_cell
                    && value.0 != 0
                {
                    *fixed = true;
                }
            }
        }

        Ok(dynamic_grid)
    }

    fn name(self) -> String {
        format!("{self:?}")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DetectAndParseReturn {
    pub detected_format: GridFormatEnum,
    pub parsed_grid: DynamicGrid,
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[enum_dispatch]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GridFormatEnum {
    BinaryCandidatesLineV0,
    BinaryCandidatesLineV1,
    BinaryCandidatesLineV2,
    CandidatesGridANSIStyled,
    CandidatesGridCompact,
    CandidatesGridPlain,
    Json,
    ValuesGrid,
    ValuesLine,
}

impl GridFormatEnum {
    pub fn all() -> Vec<Self> {
        vec![
            BinaryCandidatesLineV0.into(),
            BinaryCandidatesLineV1.into(),
            BinaryCandidatesLineV2.into(),
            CandidatesGridANSIStyled.into(),
            CandidatesGridCompact.into(),
            CandidatesGridPlain.into(),
            Json.into(),
            ValuesGrid.into(),
            ValuesLine.into(),
        ]
    }

    fn try_detect_and_parse(self, input: &str) -> Result<DetectAndParseReturn> {
        Ok(DetectAndParseReturn {
            detected_format: self,
            parsed_grid: self.parse_and_post_process(input)?,
        })
    }

    fn try_detect_and_parse_list(
        input: &str,
        formats: &[GridFormatEnum],
    ) -> Result<DetectAndParseReturn> {
        assert!(!formats.is_empty());

        let first_format = formats.first().expect("Formats list is non-empty");
        formats[1..]
            .iter()
            .fold(first_format.try_detect_and_parse(input), |acc, format| {
                acc.or_else(|_| format.try_detect_and_parse(input))
            })
    }

    pub fn detect_and_parse(input: &str) -> Result<DetectAndParseReturn> {
        let input = input.trim();
        if input.is_empty() {
            bail!("Unexpected empty input")
        }

        Self::try_detect_and_parse_list(
            input,
            &if input.starts_with('[') {
                vec![Json.into()]
            } else if input.contains('\n') {
                vec![
                    CandidatesGridANSIStyled.into(),
                    CandidatesGridCompact.into(),
                    ValuesGrid.into(),
                ]
            } else {
                vec![
                    // Contains header
                    BinaryCandidatesLineV2.into(),
                    // Comma delimited
                    BinaryCandidatesLineV0.into(),
                    // Both formats can be confused with each other.
                    // Try `ValuesLine` first, since it produces more predictable results.
                    ValuesLine.into(),
                    BinaryCandidatesLineV1.into(),
                ]
            },
        )
    }
}

impl Serialize for GridFormatEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (variant_index, variant) = match *self {
            Self::BinaryCandidatesLineV0(_) => (0, "BinaryCandidatesLineV0"),
            Self::BinaryCandidatesLineV1(_) => (1, "BinaryCandidatesLineV1"),
            Self::BinaryCandidatesLineV2(_) => (2, "BinaryCandidatesLineV2"),
            Self::CandidatesGridANSIStyled(_) => (3, "CandidatesGridANSIStyled"),
            Self::CandidatesGridCompact(_) => (4, "CandidatesGridCompact"),
            Self::CandidatesGridPlain(_) => (5, "CandidatesGridPlain"),
            Self::Json(_) => (6, "Json"),
            Self::ValuesGrid(_) => (7, "ValuesGrid"),
            Self::ValuesLine(_) => (8, "ValuesLine"),
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
    use crate::{
        cell::{Candidates, Cell, Value},
        grid::format::test_util::{
            assert_grid_format_roundtrip, assert_grid_format_roundtrip_unchanged,
        },
        test_util::{init_test_logger, test_max_base5},
    };

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

    mod via_capabilities {
        use super::*;

        // TODO: test via all sample grids

        mod cell_candidates {
            use super::*;

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

            mod lexicographical {
                use super::*;

                test_max_base5!({
                    init_test_logger();

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

            mod filled {
                use super::*;

                test_max_base5!({
                    init_test_logger();

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
    use anyhow::{Context, ensure};

    pub(crate) fn assert_parsed_grid<Base: SudokuBase>(
        expected_grid: &Grid<Base>,
        parsed_grid: &DynamicGrid,
    ) -> Result<()> {
        let parsed_grid: Grid<Base> = parsed_grid
            .clone()
            .try_into()
            .with_context(|| format!("Failed to convert cells to grid:\n{parsed_grid:#?}"))?;

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

            let parsed_grid = if detect_parse_format {
                let DetectAndParseReturn {
                    detected_format,
                    parsed_grid,
                } = GridFormatEnum::detect_and_parse(&grid_string).with_context(|| {
                    format!("Failed to detect and parse grid string:\n{grid_string}")
                })?;

                if grid_format.capabilities().detect_and_parse
                    == GridFormatDetectAndParseCapability::Detectable
                {
                    ensure!(
                        detected_format == grid_format.into(),
                        "Detected format {} does not match expected format {}",
                        detected_format.name(),
                        grid_format.name()
                    );
                }
                parsed_grid
            } else {
                grid_format
                    .parse_and_post_process(&grid_string)
                    .with_context(|| format!("Failed to parse grid string:\n{grid_string}"))?
            };

            assert_parsed_grid(expected_parsed_grid, &parsed_grid).with_context(|| {
                format!("Failed to compare parsed cells to expected parsed grid:\n{grid_string}")
            })
        })()
        .with_context(|| {
            format!(
                "Failed to roundtrip format {} {} parse detection with grid:\n{grid_to_render}",
                grid_format.name(),
                if detect_parse_format {
                    "with"
                } else {
                    "without"
                }
            )
        })
    }

    pub(crate) fn assert_grid_format_roundtrip_expected<Base: SudokuBase, F: GridFormat>(
        grid_format: F,
        grid: &Grid<Base>,
        expected_parsed_grid: &Grid<Base>,
    ) -> Result<()> {
        assert_grid_format_roundtrip(grid_format, false, grid, expected_parsed_grid)?;

        let capabilities = grid_format.capabilities();
        if let GridFormatDetectAndParseCapability::Detectable
        | GridFormatDetectAndParseCapability::DetectableViaOtherFormat =
            capabilities.detect_and_parse
        {
            assert_grid_format_roundtrip(grid_format, true, grid, expected_parsed_grid)?;
        }
        Ok(())
    }

    pub(crate) fn assert_grid_format_roundtrip_unchanged<Base: SudokuBase, F: GridFormat>(
        grid_format: F,
        grid: &Grid<Base>,
    ) -> Result<()> {
        assert_grid_format_roundtrip_expected(grid_format, grid, grid)
    }
}
