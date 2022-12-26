//! Source: https://www.sudokuoftheday.com/about/difficulty/
//!
//! Technique | Code | Cost for first use | Cost for subsequent uses
//! --- | --- | --- | ---
//! Single Candidate | sct | 100 | 100
//! Single Position | spt | 100 | 100
//! Candidate Lines | clt | 350 | 200
//! Double Pairs | dpt | 500 | 250
//! Multiple Lines | mlt | 700 | 400
//! Naked Pair | dj2 | 750 | 500
//! Hidden Pair | us2 | 1500 | 1200
//! Naked Triple | dj3 | 2000 | 1400
//! Hidden Triple | us3 | 2400 | 1600
//! X-Wing | xwg | 2800 | 1600
//! Forcing Chains | fct | 4200 | 2100
//! Naked Quad | dj4 | 5000 | 4000
//! Hidden Quad | us4 | 7000 | 5000
//! Swordfish | sf4 | 8000 | 6000

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
pub use group_reduction::GroupReduction;
pub use hidden_singles::HiddenSingles;
pub use single_candidate::SingleCandidate;

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::solver::strategic::deduction::Deductions;

// Strategies
mod backtracking;
pub mod group_reduction;
mod hidden_singles;
mod single_candidate;

#[enum_dispatch(DynamicStrategy)]
pub trait Strategy: Debug {
    /// Execute this strategy on the given grid. Returns a list of deductions.
    fn execute<Base: SudokuBase>(&self, grid: &Grid<Base>) -> Result<Deductions<Base>>;

    fn strategy_name(&self) -> String {
        format!("{:?}", self)
    }
}
#[cfg_attr(feature = "wasm", derive(TS))]
#[cfg_attr(feature = "wasm", ts(export))]
#[enum_dispatch]
#[derive(Debug)]
pub enum DynamicStrategy {
    SingleCandidate,
    HiddenSingles,
    GroupReduction,
    Backtracking,
}

impl Serialize for DynamicStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Self::SingleCandidate(_) => {
                serializer.serialize_unit_variant("Strategy", 0, "SingleCandidate")
            }
            Self::HiddenSingles(_) => {
                serializer.serialize_unit_variant("Strategy", 1, "HiddenSingles")
            }
            Self::GroupReduction(_) => {
                serializer.serialize_unit_variant("Strategy", 2, "GroupReduction")
            }
            Self::Backtracking(_) => {
                serializer.serialize_unit_variant("Strategy", 3, "Backtracking")
            }
        }
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

            fn visit_str<E>(self, strategy_name: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                strategy_name.parse().map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(StrategyVisitor)
    }
}

impl DynamicStrategy {
    pub fn all() -> Vec<Self> {
        vec![
            SingleCandidate.into(),
            HiddenSingles.into(),
            GroupReduction.into(),
            Backtracking.into(),
        ]
    }
}

impl FromStr for DynamicStrategy {
    type Err = Error;

    fn from_str(strategy_name: &str) -> Result<Self> {
        DynamicStrategy::all()
            .into_iter()
            .find(|strategy| strategy.strategy_name() == strategy_name)
            .ok_or_else(|| anyhow!("Unexpected strategy name: {strategy_name}"))
    }
}
