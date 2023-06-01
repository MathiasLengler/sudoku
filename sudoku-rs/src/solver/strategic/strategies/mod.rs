use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use anyhow::anyhow;
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "wasm")]
use ts_rs::TS;

pub use backtracking::Backtracking;
pub use group_intersection::GroupIntersection;
pub use group_reduction::GroupReduction;
pub use hidden_singles::HiddenSingles;
pub use naked_pairs::NakedPairs;
pub use naked_singles::NakedSingles;

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::solver::strategic::deduction::Deductions;

// Strategies
mod backtracking;
mod group_intersection;
pub mod group_reduction;
mod hidden_singles;
mod naked_pairs;
mod naked_singles;

#[enum_dispatch(DynamicStrategy)]
pub trait Strategy: Debug + Copy + Clone + Eq + Sized {
    /// Execute this strategy on the given grid. Returns a list of deductions.
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>>;

    fn execute_and_apply<Base: SudokuBase>(
        self,
        grid: &mut Grid<Base>,
    ) -> Result<Deductions<Base>> {
        let deductions = self.execute(grid)?;
        deductions.apply(grid)?;
        Ok(deductions)
    }

    fn name(self) -> String {
        format!("{self:?}")
    }
}
#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[enum_dispatch]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DynamicStrategy {
    NakedSingles,
    HiddenSingles,
    NakedPairs,
    GroupReduction,
    GroupIntersection,
    Backtracking,
}

impl DynamicStrategy {
    pub fn all() -> Vec<Self> {
        vec![
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            GroupReduction.into(),
            GroupIntersection.into(),
            Backtracking.into(),
        ]
    }
}

impl Serialize for DynamicStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (variant_index, variant) = match *self {
            Self::NakedSingles(_) => (0, "NakedSingles"),
            Self::HiddenSingles(_) => (1, "HiddenSingles"),
            Self::NakedPairs(_) => (2, "NakedPairs"),
            Self::GroupReduction(_) => (3, "GroupReduction"),
            Self::GroupIntersection(_) => (4, "GroupIntersection"),
            Self::Backtracking(_) => (5, "Backtracking"),
        };

        serializer.serialize_unit_variant("Strategy", variant_index, variant)
    }
}

impl<'de> Deserialize<'de> for DynamicStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrategyVisitor;

        impl<'de> Visitor<'de> for StrategyVisitor {
            type Value = DynamicStrategy;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a valid strategy name")
            }

            fn visit_str<E>(self, strategy_name: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                strategy_name.parse().map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(StrategyVisitor)
    }
}

impl FromStr for DynamicStrategy {
    type Err = Error;

    fn from_str(strategy_name: &str) -> Result<Self> {
        DynamicStrategy::all()
            .into_iter()
            .find(|strategy| strategy.name() == strategy_name)
            .ok_or_else(|| anyhow!("Unexpected strategy name: {strategy_name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_round_trip() {
        let all_strategies = DynamicStrategy::all();

        let json_string = serde_json::to_string(&all_strategies).unwrap();

        let all_strategies_round_tripped: Vec<DynamicStrategy> =
            serde_json::from_str(&json_string).unwrap();

        assert_eq!(all_strategies, all_strategies_round_tripped);
    }
}

#[cfg(test)]
mod test_util {
    use crate::base::SudokuBase;
    use crate::grid::Grid;
    use crate::solver::strategic::deduction::Deductions;

    pub(crate) fn assert_deductions<Base: SudokuBase>(
        deductions: &Deductions<Base>,
        expected_deductions: &Deductions<Base>,
    ) {
        assert_eq!(
            deductions, expected_deductions,
            "{deductions}\n!=\n{expected_deductions}"
        );
    }

    pub(crate) fn assert_deductions_with_grid<Base: SudokuBase>(
        deductions: &Deductions<Base>,
        expected_deductions: &Deductions<Base>,
        grid: &mut Grid<Base>,
    ) {
        assert_deductions(&deductions, expected_deductions);

        deductions.apply(grid).unwrap();
    }
}
