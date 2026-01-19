use super::{Strategy, impls::*};
use std::fmt::Debug;
use std::fmt::{self, Display};
use std::str::FromStr;

use anyhow::format_err;
use enum_dispatch::enum_dispatch;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Error, Result};

// TODO: struct StrategyMap<T> {
//    naked_singles: T
//    ...
// Usecases:
// - Vec<StrategyEnum> => StrategyMap<bool> (for solver)
// - stats for strategies

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
    BruteForce,
}

impl Display for StrategyEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl StrategyEnum {
    pub fn all() -> Vec<Self> {
        vec![
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            LockedSets.into(),
            GroupIntersectionBlockToAxis.into(),
            GroupIntersectionAxisToBlock.into(),
            GroupIntersectionBoth.into(),
            XWing.into(),
            BruteForce.into(),
        ]
    }

    pub fn default_solver_strategies() -> Vec<Self> {
        vec![
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            LockedSets.into(),
            GroupIntersectionBoth.into(),
            BruteForce.into(),
        ]
    }

    pub fn default_solver_strategies_no_brute_force() -> Vec<Self> {
        vec![
            NakedSingles.into(),
            HiddenSingles.into(),
            NakedPairs.into(),
            LockedSets.into(),
            GroupIntersectionBoth.into(),
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
            // LockedSets.into(),
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
        let all_strategies = StrategyEnum::all();

        let json_string = serde_json::to_string(&all_strategies).unwrap();

        let all_strategies_round_tripped: Vec<StrategyEnum> =
            serde_json::from_str(&json_string).unwrap();

        assert_eq!(all_strategies, all_strategies_round_tripped);
    }
}
