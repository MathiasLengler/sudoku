use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use anyhow::{anyhow, ensure};
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "wasm")]
use ts_rs::TS;

pub use ascii_candidates_grid::*;
pub use binary_candidates_line::*;
pub use binary_fixed_candidates_line::*;
pub use candidates_grid::*;
pub use givens_grid::*;
pub use givens_line::GivensLine;

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;

mod ascii_candidates_grid;
mod binary_candidates_line;
mod binary_fixed_candidates_line;
mod candidates_grid;
mod givens_grid;
mod givens_line;

#[enum_dispatch(DynamicGridFormat)]
pub trait GridFormat: Debug + Copy + Clone + Eq + Sized {
    fn render<Base: SudokuBase>(self, grid: &Grid<Base>) -> String;

    fn parse(self, input: &str) -> Result<Vec<DynamicCell>>;

    fn name(self) -> String {
        format!("{self:?}")
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[enum_dispatch]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DynamicGridFormat {
    GivensLine,
    GivensGrid,
    BinaryCandidatesLine,
    BinaryFixedCandidatesLine,
    CandidatesGridColored,
    CandidatesGridPlain,
    AsciiCandidatesGrid,
}

impl DynamicGridFormat {
    pub fn all() -> Vec<Self> {
        vec![
            GivensLine.into(),
            GivensGrid.into(),
            BinaryCandidatesLine.into(),
            BinaryFixedCandidatesLine.into(),
            CandidatesGridColored.into(),
            CandidatesGridPlain.into(),
            AsciiCandidatesGrid.into(),
        ]
    }

    pub fn detect_and_parse(input: &str) -> Result<Vec<DynamicCell>> {
        use crate::base::consts::ALL_CELL_COUNTS;

        let input = input.trim();

        let mut cell_views = if input.contains('\n') {
            AsciiCandidatesGrid
                .parse(input)
                .or_else(|_| GivensGrid.parse(input))?
        } else {
            GivensLine
                .parse(input)
                .or_else(|_| BinaryCandidatesLine.parse(input))?
        };

        let actual_cell_count = cell_views.len().try_into()?;

        ensure!(
            ALL_CELL_COUNTS.contains(&actual_cell_count),
            "Unexpected cell count {actual_cell_count}, expected one of: {ALL_CELL_COUNTS:?}"
        );

        // Fix all values
        for cell_view in &mut cell_views {
            if let DynamicCell::Value { fixed, value } = cell_view {
                if value.0 != 0 {
                    *fixed = true;
                }
            }
        }

        Ok(cell_views)
    }
}

impl Serialize for DynamicGridFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (variant_index, variant) = match *self {
            Self::GivensLine(_) => (0, "GivensLine"),
            Self::GivensGrid(_) => (1, "GivensGrid"),
            Self::BinaryCandidatesLine(_) => (2, "BinaryCandidatesLine"),
            Self::BinaryFixedCandidatesLine(_) => (3, "BinaryFixedCandidatesLine"),
            Self::CandidatesGridColored(_) => (4, "CandidatesGridColored"),
            Self::CandidatesGridPlain(_) => (5, "CandidatesGridPlain"),
            Self::AsciiCandidatesGrid(_) => (6, "AsciiCandidatesGrid"),
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
            .ok_or_else(|| anyhow!("Unexpected grid format name: {grid_format_name}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    #[test]
    fn test_detect_and_parse_cells_roundtrip() {
        fn assert_grid_roundtrip<Base: SudokuBase>(
            grid_format: DynamicGridFormat,
            grid_string: &str,
            grid: &Grid<Base>,
        ) {
            use anyhow::Context;

            let cell_views = DynamicGridFormat::detect_and_parse(grid_string)
                .with_context(|| {
                    format!(
                        "parse_cells to parse:\n\
                        {grid_string}"
                    )
                })
                .unwrap();

            let parsed_grid: Grid<Base> = cell_views
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
                grid, &parsed_grid,
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

        let grid_formats: Vec<DynamicGridFormat> = vec![
            GivensLine.into(),
            GivensGrid.into(),
            BinaryCandidatesLine.into(),
            // FIXME: handle formats in parse_cells
            // GridFormat::CandidatesGrid,
        ];

        for grid_format in grid_formats {
            for (grid_string, grid) in samples::base_2()
                .into_iter()
                .map(|grid| (grid_format.render(&grid), grid))
            {
                assert_grid_roundtrip(grid_format, &grid_string, &grid);
            }

            for (grid_string, grid) in samples::base_3()
                .into_iter()
                .map(|grid| (grid_format.render(&grid), grid))
            {
                assert_grid_roundtrip(grid_format, &grid_string, &grid);
            }
        }
    }
}
