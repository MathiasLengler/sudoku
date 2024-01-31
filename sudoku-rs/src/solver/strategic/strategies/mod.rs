use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use anyhow::format_err;
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "wasm")]
use ts_rs::TS;

pub use backtracking::Backtracking;
pub use group_intersection::{
    GroupIntersectionAxisToBlock, GroupIntersectionBlockToAxis, GroupIntersectionBoth,
};
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
    /// The name of the strategy.
    fn name(self) -> &'static str;

    /// Execute this strategy on the given grid. Returns a list of deductions.
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>>;

    /// Execute this strategy on the given grid and applies the deductions to it.
    /// Returns a list of applied deductions.
    fn execute_and_apply<Base: SudokuBase>(
        self,
        grid: &mut Grid<Base>,
    ) -> Result<Deductions<Base>> {
        let deductions = self.execute(grid)?;
        deductions.apply(grid)?;
        Ok(deductions)
    }
}
#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[enum_dispatch]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum DynamicStrategy {
    NakedSingles,
    HiddenSingles,
    NakedPairs,
    GroupReduction,
    GroupIntersectionBlockToAxis,
    GroupIntersectionAxisToBlock,
    GroupIntersectionBoth,
    Backtracking,
}

impl DynamicStrategy {
    pub fn all() -> Vec<Self> {
        vec![
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            GroupReduction.into(),
            GroupIntersectionBlockToAxis.into(),
            GroupIntersectionAxisToBlock.into(),
            GroupIntersectionBoth.into(),
            Backtracking.into(),
        ]
    }

    pub fn default_solver_strategies() -> Vec<Self> {
        vec![
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            GroupReduction.into(),
            GroupIntersectionBoth.into(),
            Backtracking.into(),
        ]
    }

    pub fn introspective_solver_base_4_plus_strategies() -> Vec<Self> {
        // TODO: benchmark
        vec![
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            // FIXME: Slow for empty groups
            //  also slow for base 3, but impact is worse for larger bases
            // GroupReduction.into(),
            GroupIntersectionBoth.into(),
        ]
    }

    fn variant_index(&self) -> u32 {
        // Reference: https://doc.rust-lang.org/std/mem/fn.discriminant.html

        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }.into()
    }
}

impl Serialize for DynamicStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_variant("Strategy", self.variant_index(), self.name())
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
            .ok_or_else(|| format_err!("Unexpected strategy name: {strategy_name}"))
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
        assert_deductions(deductions, expected_deductions);

        deductions.apply(grid).unwrap();
    }
}
