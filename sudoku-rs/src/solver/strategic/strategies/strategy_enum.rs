use super::{Strategy, impls::*};
use crate::error::{Error, Result};
use crate::solver::strategic::strategies::map::StrategyMap;
use anyhow::format_err;
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Debug;
use std::fmt::{self, Display};
use std::str::FromStr;

const STRATEGY_COUNT: usize = 10;

pub mod map {
    use super::*;
    use crate::solver::strategic::strategies::StrategyScore;

    /// A map of `StrategyEnum` to `T`.
    #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct StrategyMap<T> {
        pub naked_singles: T,
        pub hidden_singles: T,
        pub naked_pairs: T,
        pub locked_sets: T,
        pub group_intersection_block_to_axis: T,
        pub group_intersection_axis_to_block: T,
        pub group_intersection_both: T,
        pub x_wing: T,
        pub bug: T,
        pub brute_force: T,
    }

    impl<T> StrategyMap<T> {
        pub const fn with_all(value: T) -> Self
        where
            T: Copy,
        {
            Self {
                naked_singles: value,
                hidden_singles: value,
                naked_pairs: value,
                locked_sets: value,
                group_intersection_block_to_axis: value,
                group_intersection_axis_to_block: value,
                group_intersection_both: value,
                x_wing: value,
                bug: value,
                brute_force: value,
            }
        }

        pub const fn get(&self, strategy: StrategyEnum) -> &T {
            match strategy {
                StrategyEnum::NakedSingles(_) => &self.naked_singles,
                StrategyEnum::HiddenSingles(_) => &self.hidden_singles,
                StrategyEnum::NakedPairs(_) => &self.naked_pairs,
                StrategyEnum::LockedSets(_) => &self.locked_sets,
                StrategyEnum::GroupIntersectionBlockToAxis(_) => {
                    &self.group_intersection_block_to_axis
                }
                StrategyEnum::GroupIntersectionAxisToBlock(_) => {
                    &self.group_intersection_axis_to_block
                }
                StrategyEnum::GroupIntersectionBoth(_) => &self.group_intersection_both,
                StrategyEnum::XWing(_) => &self.x_wing,
                StrategyEnum::Bug(_) => &self.bug,
                StrategyEnum::BruteForce(_) => &self.brute_force,
            }
        }
        pub const fn get_mut(&mut self, strategy: StrategyEnum) -> &mut T {
            match strategy {
                StrategyEnum::NakedSingles(_) => &mut self.naked_singles,
                StrategyEnum::HiddenSingles(_) => &mut self.hidden_singles,
                StrategyEnum::NakedPairs(_) => &mut self.naked_pairs,
                StrategyEnum::LockedSets(_) => &mut self.locked_sets,
                StrategyEnum::GroupIntersectionBlockToAxis(_) => {
                    &mut self.group_intersection_block_to_axis
                }
                StrategyEnum::GroupIntersectionAxisToBlock(_) => {
                    &mut self.group_intersection_axis_to_block
                }
                StrategyEnum::GroupIntersectionBoth(_) => &mut self.group_intersection_both,
                StrategyEnum::XWing(_) => &mut self.x_wing,
                StrategyEnum::Bug(_) => &mut self.bug,
                StrategyEnum::BruteForce(_) => &mut self.brute_force,
            }
        }

        pub fn into_values(self) -> [T; STRATEGY_COUNT] {
            [
                self.naked_singles,
                self.hidden_singles,
                self.naked_pairs,
                self.locked_sets,
                self.group_intersection_block_to_axis,
                self.group_intersection_axis_to_block,
                self.group_intersection_both,
                self.x_wing,
                self.bug,
                self.brute_force,
            ]
        }
    }

    impl<T> IntoIterator for StrategyMap<T> {
        type Item = (StrategyEnum, T);

        type IntoIter = std::iter::Zip<
            std::array::IntoIter<StrategyEnum, STRATEGY_COUNT>,
            std::array::IntoIter<T, STRATEGY_COUNT>,
        >;

        fn into_iter(self) -> Self::IntoIter {
            StrategyEnum::all().into_iter().zip(self.into_values())
        }
    }

    impl Default for StrategyMap<StrategyScore> {
        fn default() -> Self {
            Self::with_all(0)
        }
    }
}

pub mod selection {
    use super::*;

    /// A selection of strategies in canonical order.
    pub type StrategySet = StrategyMap<bool>;
    /// A selection of strategies in a specific order.
    pub type StrategyList = Vec<StrategyEnum>;

    /// A selection of strategies.
    ///
    /// Primarily used specify the strategies used by `strategic::Solver`.
    pub trait StrategySelection {
        fn iter_strategies(&self) -> impl Iterator<Item = StrategyEnum>;
    }

    impl StrategySelection for StrategySet {
        /// Iterates over the selected strategies in "canonical" (`StrategyEnum::all()`) order.
        fn iter_strategies(&self) -> impl Iterator<Item = StrategyEnum> {
            self.into_iter()
                .filter_map(|(strategy, selected)| selected.then_some(strategy))
        }
    }

    impl StrategySelection for StrategyList {
        /// Iterates over the selected strategies in the order they are stored in the vec.
        fn iter_strategies(&self) -> impl Iterator<Item = StrategyEnum> {
            self.iter().copied()
        }
    }

    impl Default for StrategySet {
        fn default() -> Self {
            Self::default_solver_strategies()
        }
    }

    impl StrategySet {
        pub const fn with_single(strategy: StrategyEnum) -> Self {
            let mut this = Self::with_all(false);
            *this.get_mut(strategy) = true;
            this
        }

        pub fn count(&self) -> usize {
            self.into_values()
                .into_iter()
                .map(Into::<usize>::into)
                .sum()
        }

        pub fn is_empty(&self) -> bool {
            self.count() == 0
        }

        pub const fn default_solver_strategies() -> Self {
            StrategySet {
                naked_singles: true,
                hidden_singles: true,
                naked_pairs: true,
                locked_sets: true,
                group_intersection_both: true,
                x_wing: true,
                brute_force: true,
                ..StrategySet::with_all(false)
            }
        }

        pub fn default_solver_strategies_no_brute_force() -> Self {
            let mut strategies = Self::default_solver_strategies();
            strategies.brute_force = false;
            strategies
        }
    }

    impl FromIterator<StrategyEnum> for StrategySet {
        fn from_iter<TIter: IntoIterator<Item = StrategyEnum>>(iter: TIter) -> Self {
            let mut this = StrategySet::with_all(false);
            for strategy in iter {
                *this.get_mut(strategy) = true;
            }
            this
        }
    }
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[enum_dispatch]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum StrategyEnum {
    NakedSingles,
    HiddenSingles,
    NakedPairs,
    LockedSets,
    GroupIntersectionBlockToAxis,
    GroupIntersectionAxisToBlock,
    GroupIntersectionBoth,
    XWing,
    Bug,
    BruteForce,
}

impl Display for StrategyEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl StrategyEnum {
    pub fn all() -> [Self; STRATEGY_COUNT] {
        [
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            LockedSets.into(),
            GroupIntersectionBlockToAxis.into(),
            GroupIntersectionAxisToBlock.into(),
            GroupIntersectionBoth.into(),
            XWing.into(),
            Bug.into(),
            BruteForce.into(),
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

impl Serialize for StrategyEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_variant("Strategy", self.variant_index(), self.name())
    }
}

impl<'de> Deserialize<'de> for StrategyEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrategyVisitor;

        impl Visitor<'_> for StrategyVisitor {
            type Value = StrategyEnum;

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

impl FromStr for StrategyEnum {
    type Err = Error;

    fn from_str(strategy_name: &str) -> Result<Self> {
        StrategyEnum::all()
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
        let all_strategies = StrategyEnum::all().to_vec();

        let json_string = serde_json::to_string(&all_strategies).unwrap();

        let all_strategies_round_tripped: Vec<StrategyEnum> =
            serde_json::from_str(&json_string).unwrap();

        assert_eq!(all_strategies, all_strategies_round_tripped);
    }
}
