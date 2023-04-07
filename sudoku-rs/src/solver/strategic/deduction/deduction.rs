use std::fmt::{Display, Formatter};

use anyhow::ensure;
use itertools::Itertools;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::index::position::Position;
use crate::grid::Grid;
use crate::solver::strategic::deduction::{Action, PositionMap, Reason};

// TODO: easier instantiation of Deduction for test

/// A single, self-contained result of a strategy.
/// Consists of actions to be taken on a Sudoku grid, as well as the reasons why.
/// # Examples
/// - a single hidden single
/// - a single pair
/// - a single X-Wing
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Deduction<Base: SudokuBase> {
    pub actions: PositionMap<Base, Action<Base>>,
    pub reasons: PositionMap<Base, Reason<Base>>,
}

impl<Base: SudokuBase> Display for Deduction<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, because of: {}",
            self.actions
                .iter()
                .map(|(pos, action)| format!("{pos}: {action}"))
                .join(", "),
            self.reasons
                .iter()
                .map(|(pos, reason)| format!("{pos}: {reason}"))
                .join(", ")
        )
    }
}

impl<Base: SudokuBase> Default for Deduction<Base> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Base: SudokuBase> Deduction<Base> {
    pub fn new() -> Self {
        Self {
            actions: PositionMap::new(),
            reasons: PositionMap::new(),
        }
    }

    pub fn with_action(pos: Position<Base>, action: Action<Base>) -> Self {
        Self {
            actions: PositionMap::with_single(pos, action),
            ..Default::default()
        }
    }

    pub fn try_from_actions(
        actions: impl Iterator<Item = (Position<Base>, Action<Base>)>,
    ) -> Result<Self> {
        Ok(Self {
            actions: PositionMap::try_from_iter(actions)?,
            ..Default::default()
        })
    }

    pub fn try_from_iters(
        reasons: impl Iterator<Item = (Position<Base>, Reason<Base>)>,
        actions: impl Iterator<Item = (Position<Base>, Action<Base>)>,
    ) -> Result<Self> {
        Ok(Self {
            reasons: PositionMap::try_from_iter(reasons)?,
            actions: PositionMap::try_from_iter(actions)?,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty() && self.reasons.is_empty()
    }

    pub fn validate(&self, grid: &Grid<Base>) -> Result<()> {
        ensure!(
            !self.actions.is_empty(),
            "expected deduction to contain at least one action"
        );

        for (pos, action) in &self.actions {
            action.validate(grid.get(pos))?;
        }

        for (pos, reason) in &self.reasons {
            reason.validate(grid.get(pos))?;
        }

        // TODO: validate that actions and reasons are not in conflict, e.g. for the same position:
        //  - SetValue and Reason
        //  - DeleteCandidate and Reason share candidate

        Ok(())
    }

    pub fn apply(&self, grid: &mut Grid<Base>) -> Result<()> {
        self.validate(grid)?;

        for (pos, action) in &self.actions {
            action.apply(grid.get_mut(pos))?;
        }

        // Update candidates for all set value actions.
        for (pos, action) in &self.actions {
            action.update_direct_candidates(grid, pos);
        }

        Ok(())
    }
}
