use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use anyhow::{bail, ensure, format_err};
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "wasm")]
use ts_rs::TS;

pub use binary_candidates_line::*;
pub use binary_fixed_candidates_line::*;
pub use candidates_grid::*;
pub use candidates_grid_compact::*;
pub use givens_grid::*;
pub use givens_line::GivensLine;

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;

mod binary_candidates_line;
mod binary_fixed_candidates_line;
mod candidates_grid;
mod candidates_grid_compact;
mod givens_grid;
mod givens_line;

#[enum_dispatch(DynamicGridFormat)]
pub trait GridFormat: Debug + Copy + Clone + Eq + Sized {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String;

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>>;

    fn do_fix_all_values(self) -> bool {
        true
    }

    fn parse_and_validate_cell_count(self, input: &str) -> Result<Vec<DynamicCell>> {
        use crate::base::consts::ALL_CELL_COUNTS;

        let mut dynamic_cells = GridFormat::parse(self, input)?;

        let actual_cell_count = dynamic_cells.len().try_into()?;

        ensure!(
            ALL_CELL_COUNTS.contains(&actual_cell_count),
            "Unexpected cell count {actual_cell_count}, expected one of: {ALL_CELL_COUNTS:?}"
        );

        if self.do_fix_all_values() {
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

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[enum_dispatch]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DynamicGridFormat {
    BinaryCandidatesLine,
    BinaryFixedCandidatesLine,
    CandidatesGridANSIStyled,
    CandidatesGridPlain,
    CandidatesGridCompact,
    GivensLine,
    GivensGrid,
}

impl DynamicGridFormat {
    pub fn all() -> Vec<Self> {
        vec![
            BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            CandidatesGridANSIStyled.into(),
            CandidatesGridPlain.into(),
            CandidatesGridCompact.into(),
            GivensLine.into(),
            GivensGrid.into(),
        ]
    }

    pub fn detect_and_parse(input: &str) -> Result<Vec<DynamicCell>> {
        let input = input.trim();
        if input.is_empty() {
            bail!("Unexpected empty input")
        }

        let cell_views = if input.contains('\n') {
            // TODO: add CandidatesGridANSIStyled

            CandidatesGridANSIStyled
                .parse_and_validate_cell_count(input)
                .or_else(|_| CandidatesGridCompact.parse_and_validate_cell_count(input))
                .or_else(|_| GivensGrid.parse_and_validate_cell_count(input))?
        } else {
            GivensLine
                .parse_and_validate_cell_count(input)
                .or_else(|_| BinaryFixedCandidatesLine.parse_and_validate_cell_count(input))
                .or_else(|_| BinaryCandidatesLine.parse_and_validate_cell_count(input))?
        };

        Ok(cell_views)
    }
}

impl Serialize for DynamicGridFormat {
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
            Self::GivensLine(_) => (5, "GivensLine"),
            Self::GivensGrid(_) => (6, "GivensGrid"),
        };

        serializer.serialize_unit_variant("Strategy", variant_index, variant)
    }
}

impl<'de> Deserialize<'de> for DynamicGridFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GridFormatVisitor;

        impl<'de> Visitor<'de> for GridFormatVisitor {
            type Value = DynamicGridFormat;

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

impl FromStr for DynamicGridFormat {
    type Err = Error;

    fn from_str(grid_format_name: &str) -> Result<Self> {
        DynamicGridFormat::all()
            .into_iter()
            .find(|grid_format| grid_format.name() == grid_format_name)
            .ok_or_else(|| format_err!("Unexpected grid format name: {grid_format_name}"))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use crate::samples;

    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let all_strategies = DynamicGridFormat::all();

        let json_string = serde_json::to_string(&all_strategies).unwrap();

        let all_strategies_round_tripped: Vec<DynamicGridFormat> =
            serde_json::from_str(&json_string).unwrap();

        assert_eq!(all_strategies, all_strategies_round_tripped);
    }

    #[test]
    fn test_detect_and_parse_cells_roundtrip() {
        pub(crate) fn assert_grid_format_roundtrip_detect<Base: SudokuBase>(
            grid: &Grid<Base>,
            grid_format: DynamicGridFormat,
        ) -> Result<()> {
            (|| {
                let grid_string = grid_format.render(grid);

                let cell_views =
                    DynamicGridFormat::detect_and_parse(&grid_string).with_context(|| {
                        format!("Failed to detect and parse grid_string:\n{grid_string}")
                    })?;

                test_util::assert_grid_equals_cell_views(grid, &cell_views).with_context(|| {
                    format!("Failed to compare cell views to grid for grid_string:\n{grid_string}")
                })
            })()
            .with_context(|| {
                format!(
                    "Failed to roundtrip format {} with grid:\n{grid}",
                    grid_format.name()
                )
            })
        }

        // Grid formats which preserve:
        // - cell value
        let grid_formats: Vec<DynamicGridFormat> = vec![
            BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            CandidatesGridANSIStyled.into(),
            CandidatesGridPlain.into(),
            CandidatesGridCompact.into(),
            GivensLine.into(),
            GivensGrid.into(),
        ];

        // TODO: refactor other duplicated base handling code
        macro_rules! for_test_grids {
            (|$grid:ident| $block:block) => {
                #[allow(unused_mut)]
                for mut $grid in samples::base_2() $block
                #[allow(unused_mut)]
                for mut $grid in samples::base_3() $block
            };
        }

        for grid_format in grid_formats {
            for_test_grids!(|grid| {
                grid.fix_all_values();

                assert_grid_format_roundtrip_detect(&grid, grid_format)
                    .with_context(|| "Test cell value roundtrip".to_string())
                    .unwrap();
            });
        }

        // Grid formats which preserve:
        // - cell value
        // - cell value fixed state
        let grid_formats: Vec<DynamicGridFormat> = vec![
            // BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            // CandidatesGridANSIStyled.into(),
            // CandidatesGridPlain.into(),
            // CandidatesGridCompact.into(),
            // GivensLine.into(),
            // GivensGrid.into(),
        ];

        for grid_format in grid_formats {
            for_test_grids!(|grid| {
                grid.fix_all_values();
                assert_grid_format_roundtrip_detect(&grid, grid_format)
                    .with_context(|| "Test cell fixed value roundtrip".to_string())
                    .unwrap();
                grid.unfix_all_values();
                assert_grid_format_roundtrip_detect(&grid, grid_format)
                    .with_context(|| "Test cell unfixed value roundtrip".to_string())
                    .unwrap();
            });
        }

        // Grid formats which preserve:
        // - cell value
        // - cell candidates
        let grid_formats: Vec<DynamicGridFormat> = vec![
            // BinaryCandidatesLine.into(),
            // BinaryFixedCandidatesLine.into(),
            CandidatesGridANSIStyled.into(),
            CandidatesGridPlain.into(),
            CandidatesGridCompact.into(),
            // GivensLine.into(),
            // GivensGrid.into(),
        ];
        for grid_format in grid_formats {
            for_test_grids!(|grid| {
                assert_grid_format_roundtrip_detect(&grid, grid_format)
                    .with_context(|| "Test cell candidates roundtrip".to_string())
                    .unwrap();
            });
        }

        // Grid formats which preserve:
        // - cell value
        // - cell value fixed state
        // - cell multiple candidates
        let grid_formats: Vec<DynamicGridFormat> = vec![
            // BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            // CandidatesGridANSIStyled.into(),
            // CandidatesGridPlain.into(),
            // CandidatesGridCompact.into(),
            // GivensLine.into(),
            // GivensGrid.into(),
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
                assert_grid_format_roundtrip_detect(&grid, grid_format)
                    .with_context(|| "Test cell multiple candidates roundtrip".to_string())
                    .unwrap();
            });
        }
    }
}

#[cfg(test)]
mod test_util {
    use anyhow::Context;

    use super::*;

    pub(crate) fn assert_grid_equals_cell_views<Base: SudokuBase>(
        grid: &Grid<Base>,
        cell_views: &[DynamicCell],
    ) -> Result<()> {
        let parsed_grid: Grid<Base> = cell_views
            .to_vec()
            .try_into()
            .with_context(|| format!("Failed to convert cell_views to grid:\n{cell_views:#?}"))?;

        ensure!(grid == &parsed_grid, "Mismatched grid:\n{parsed_grid}");

        Ok(())
    }

    pub(crate) fn assert_grid_format_roundtrip<Base: SudokuBase, F: GridFormat>(
        grid: &Grid<Base>,
        grid_format: F,
    ) -> Result<()> {
        (|| {
            let grid_string = grid_format.render(grid);

            let cell_views = grid_format
                .parse(&grid_string)
                .with_context(|| format!("Failed to parse grid_string:\n{grid_string}"))?;

            assert_grid_equals_cell_views(grid, &cell_views)
        })()
        .with_context(|| format!("Failed to roundtrip format {grid_format:?} with grid:\n{grid}"))
    }
}
