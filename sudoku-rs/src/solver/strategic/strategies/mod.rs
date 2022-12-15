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

use std::fmt::Debug;
use std::str::FromStr;

use anyhow::anyhow;
use enum_dispatch::enum_dispatch;

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

#[enum_dispatch]
#[derive(Debug)]
pub enum DynamicStrategy {
    SingleCandidate,
    HiddenSingles,
    GroupReduction,
    Backtracking,
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
