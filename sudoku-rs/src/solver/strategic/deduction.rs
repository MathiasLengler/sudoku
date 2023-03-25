use std::collections::btree_map::Iter;
use std::collections::{btree_map, btree_set, BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::iter::Map;

use anyhow::{bail, ensure, Context};
use itertools::Itertools;
use serde::Serialize;
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::cell_state::CellState;
use crate::cell::compact::value::Value;
use crate::cell::Cell;
use crate::error::Result;
use crate::grid::index::position::Position;
use crate::grid::Grid;

/// A list of deductions made by a strategy.
/// Some strategies can be applied multiple times on a single grid, e.g.:
/// - multiple hidden singles
/// - multiple pairs
/// - multiple distinct X-Wings
///
/// Strategies are encouraged to report logically separate deductions as multiple instances of `Deduction`,
/// in order to enable:
/// - application of single `Deductions`
/// - clear distinction of the reasoning for each `Deduction`
/// - enabling the hint UI to only reveal a single `Deduction`
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Deductions<Base: SudokuBase> {
    deductions: BTreeSet<Deduction<Base>>,
}

impl<Base: SudokuBase> Display for Deductions<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.deductions
                .iter()
                .map(|deduction| deduction.to_string())
                .join("\n")
        )
    }
}

impl<Base: SudokuBase> Deductions<Base> {
    pub fn iter(&self) -> btree_set::Iter<'_, Deduction<Base>> {
        self.deductions.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.deductions.is_empty()
    }

    fn as_merged_deduction(&self) -> Result<Deduction<Base>> {
        let mut merged_deduction = Deduction::default();

        for deduction in &self.deductions {
            for (pos, action) in deduction.actions.iter() {
                merged_deduction.actions.insert(pos, *action)?;
            }
            for (pos, reasons) in deduction.reasons.iter() {
                merged_deduction.reasons.insert(pos, *reasons)?;
            }
        }
        Ok(merged_deduction)
    }

    /// If two deductions contain the same reasons, merge them into a single deduction by merging their actions.
    pub fn merge_deductions_by_reasons(self) -> Result<Self> {
        let mut reasons_to_actions: BTreeMap<
            PositionMap<Base, Reason<Base>>,
            PositionMap<Base, Action<Base>>,
        > = BTreeMap::new();

        for Deduction { reasons, actions } in self {
            if let Some(existing_actions) = reasons_to_actions.get_mut(&reasons) {
                existing_actions.merge(actions)?;
            } else {
                reasons_to_actions.insert(reasons, actions);
            }
        }

        Ok(reasons_to_actions
            .into_iter()
            .map(|(reasons, actions)| Deduction { actions, reasons })
            .collect())
    }

    fn validate(&self, grid: &Grid<Base>) -> Result<()> {
        for deduction in &self.deductions {
            deduction.validate(grid)?;
        }
        Ok(())
    }

    pub fn apply(&self, grid: &mut Grid<Base>) -> Result<()> {
        self.validate(grid)?;

        let merged_deduction = self.as_merged_deduction()?;
        merged_deduction.apply(grid)?;

        Ok(())
    }
}

impl<'a, Base: SudokuBase> IntoIterator for &'a Deductions<Base> {
    type Item = &'a Deduction<Base>;
    type IntoIter = btree_set::Iter<'a, Deduction<Base>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Base: SudokuBase> IntoIterator for Deductions<Base> {
    type Item = Deduction<Base>;
    type IntoIter = btree_set::IntoIter<Deduction<Base>>;

    fn into_iter(self) -> Self::IntoIter {
        self.deductions.into_iter()
    }
}

impl<Base: SudokuBase> FromIterator<Deduction<Base>> for Deductions<Base> {
    fn from_iter<T: IntoIterator<Item = Deduction<Base>>>(iter: T) -> Self {
        Self {
            deductions: iter.into_iter().collect(),
        }
    }
}

pub trait Merge: Sized {
    fn merge(&mut self, other: Self) -> Result<()>;
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PositionMap<Base: SudokuBase, T: Merge> {
    map: BTreeMap<Position<Base>, T>,
}

impl<Base: SudokuBase, T: Merge> Merge for PositionMap<Base, T> {
    fn merge(&mut self, other: Self) -> Result<()> {
        for (pos, value) in other {
            self.insert(pos, value)?;
        }
        Ok(())
    }
}

impl<Base: SudokuBase, T: Merge> Default for PositionMap<Base, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Base: SudokuBase, T: Merge> IntoIterator for PositionMap<Base, T> {
    type Item = (Position<Base>, T);
    type IntoIter = btree_map::IntoIter<Position<Base>, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

type PositionMapIter<'a, Base, T> =
    Map<Iter<'a, Position<Base>, T>, fn((&Position<Base>, &'a T)) -> (Position<Base>, &'a T)>;

impl<'a, Base: SudokuBase, T: Merge> IntoIterator for &'a PositionMap<Base, T> {
    type Item = (Position<Base>, &'a T);
    type IntoIter = PositionMapIter<'a, Base, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Base: SudokuBase, T: Merge> PositionMap<Base, T> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::default(),
        }
    }

    pub fn with_single(pos: Position<Base>, value: T) -> Self {
        let mut this: Self = Self::new();
        this.map.insert(pos, value);
        this
    }

    pub fn try_from_iter(iter: impl Iterator<Item = (Position<Base>, T)>) -> Result<Self> {
        let mut this = Self::new();

        for (pos, value) in iter {
            this.insert(pos, value)?;
        }

        Ok(this)
    }

    // False positive
    pub fn iter(&self) -> PositionMapIter<'_, Base, T> {
        self.map.iter().map(|(pos, value)| (*pos, value))
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn insert(&mut self, pos: Position<Base>, value: T) -> Result<()> {
        if let Some(existing_value) = self.map.get_mut(&pos) {
            existing_value.merge(value)?;
        } else {
            self.map.insert(pos, value);
        }

        Ok(())
    }
}

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
            actions: PositionMap::with_single(pos.into(), action),
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

    fn validate(&self, grid: &Grid<Base>) -> Result<()> {
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

    fn apply(&self, grid: &mut Grid<Base>) -> Result<()> {
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

/// On what basis a deduction was made.
/// Used to highlight/explain a deduction in the UI.
#[cfg_attr(
    feature = "wasm",
    derive(TS),
    ts(export, export_generic_params = "crate::base::consts::Base3")
)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(bound = "", rename_all = "camelCase")]
pub enum Reason<Base: SudokuBase> {
    Candidate { candidate: Value<Base> },
    Candidates { candidates: Candidates<Base> },
}

impl<Base: SudokuBase> Display for Reason<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::Candidate { candidate } => {
                write!(f, "candidate {candidate}")
            }
            Reason::Candidates { candidates } => {
                write!(f, "candidates {candidates}")
            }
        }
    }
}

impl<Base: SudokuBase> Reason<Base> {
    fn validate(&self, cell: &Cell<Base>) -> Result<()> {
        (|| {
            match cell.state() {
                CellState::Value(value) | CellState::FixedValue(value) => {
                    bail!("unexpected cell with value {value}")
                }
                CellState::Candidates(existing_candidates) => match *self {
                    Reason::Candidate { candidate } => {
                        ensure!(
                            existing_candidates.has(candidate),
                            "candidate {candidate} is missing from cell candidates {existing_candidates}"
                        );
                    }
                    Reason::Candidates { candidates } => {
                        ensure!(!candidates.is_empty(), "candidates must not be empty");
                        let unexpected_candidates = candidates.without(*existing_candidates);
                        ensure!(unexpected_candidates.is_empty(), "unexpected candidates {unexpected_candidates}");
                    }
                },
            }
            Ok(())
        })()
        .with_context(|| format!("Invalid reason {self} for cell {cell}"))
    }
}

impl<Base: SudokuBase> Merge for Reason<Base> {
    fn merge(&mut self, other: Self) -> Result<()> {
        *self = match (*self, other) {
            (
                Reason::Candidate {
                    candidate: self_candidate,
                },
                Reason::Candidate {
                    candidate: other_candidate,
                },
            ) => {
                if self_candidate == other_candidate {
                    Reason::Candidate {
                        candidate: self_candidate,
                    }
                } else {
                    Reason::Candidates {
                        candidates: Candidates::from(vec![self_candidate, other_candidate]),
                    }
                }
            }
            (
                Reason::Candidates { candidates },
                Reason::Candidates {
                    candidates: other_candidates,
                },
            ) => Reason::Candidates {
                candidates: candidates.union(other_candidates),
            },
            (Reason::Candidate { candidate }, Reason::Candidates { candidates })
            | (Reason::Candidates { candidates }, Reason::Candidate { candidate }) => {
                Reason::Candidates {
                    candidates: candidates.union(Candidates::single(candidate)),
                }
            }
        };

        Ok(())
    }
}

/// What action should be taken for a specific cell.
#[cfg_attr(
    feature = "wasm",
    derive(TS),
    ts(export, export_generic_params = "crate::base::consts::Base3")
)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(bound = "", rename_all = "camelCase")]
pub enum Action<Base: SudokuBase> {
    SetValue { value: Value<Base> },
    DeleteCandidate { candidate: Value<Base> },
    DeleteCandidates { candidates: Candidates<Base> },
}

impl<Base: SudokuBase> Display for Action<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::SetValue { value } => {
                write!(f, "set value {value}")
            }
            Action::DeleteCandidate { candidate } => {
                write!(f, "delete candidate {candidate}")
            }
            Action::DeleteCandidates { candidates } => {
                write!(f, "delete candidates {candidates}")
            }
        }
    }
}

impl<Base: SudokuBase> Action<Base> {
    fn validate(&self, cell: &Cell<Base>) -> Result<Candidates<Base>> {
        (|| {
            let Some(existing_candidates) = cell.candidates() else {
                bail!("expected cell to contain candidates")
            };
            match *self {
                Action::SetValue { value } => {
                    ensure!(
                        existing_candidates.has(value),
                        "expected cell to contain the candidate {value}"
                    );
                }
                Action::DeleteCandidate { candidate } => {
                    ensure!(
                        existing_candidates.has(candidate),
                        "expected cell to contain the candidate {candidate}"
                    );
                    ensure!(
                        existing_candidates.count() > 1,
                        "can't delete last candidate {candidate}"
                    );
                }
                Action::DeleteCandidates { candidates } => {
                    ensure!(
                        candidates.without(existing_candidates).is_empty(),
                        "expected cell to contain the candidates {candidates}"
                    );
                    let remaining_candidates = existing_candidates.without(candidates);
                    ensure!(
                        !remaining_candidates.is_empty(),
                        "can't delete all candidates {candidates}"
                    );
                }
            }
            Ok(existing_candidates)
        })()
        .with_context(|| format!("Invalid action {self} for cell {cell}"))
    }

    fn apply(&self, cell: &mut Cell<Base>) -> Result<()> {
        let existing_candidates = self.validate(cell)?;
        match *self {
            Action::SetValue { value } => {
                cell.set_value(value);
                // Defer updating of direct candidates
            }
            Action::DeleteCandidate { candidate } => {
                cell.delete_candidate(candidate);
            }
            Action::DeleteCandidates { candidates } => {
                cell.set_candidates(existing_candidates.without(candidates));
            }
        }

        Ok(())
    }

    fn update_direct_candidates(&self, grid: &mut Grid<Base>, pos: Position<Base>) {
        if let Action::SetValue { value } = self {
            grid.update_direct_candidates(pos, *value);
        }
    }
}

impl<Base: SudokuBase> Merge for Action<Base> {
    fn merge(&mut self, other: Self) -> Result<()> {
        use Action::*;
        *self = (|| {
            Ok(match (*self, other) {
                (SetValue { value: self_value }, SetValue { value: other_value }) => {
                    ensure!(
                        self_value == other_value,
                        "conflicting values: {self_value} != {other_value}"
                    );
                    SetValue { value: self_value }
                }
                (
                    DeleteCandidate {
                        candidate: self_candidate,
                    },
                    DeleteCandidate {
                        candidate: other_candidate,
                    },
                ) => {
                    if self_candidate == other_candidate {
                        DeleteCandidate {
                            candidate: self_candidate,
                        }
                    } else {
                        DeleteCandidates {
                            candidates: Candidates::from(vec![self_candidate, other_candidate]),
                        }
                    }
                }
                (
                    DeleteCandidates {
                        candidates: self_candidates,
                    },
                    DeleteCandidates {
                        candidates: other_candidates,
                    },
                ) => DeleteCandidates {
                    candidates: self_candidates.union(other_candidates),
                },
                (SetValue { value }, DeleteCandidate { candidate })
                | (DeleteCandidate { candidate }, SetValue { value }) => {
                    ensure!(
                        value != candidate,
                        "can't set deleted candidate {value} as a value"
                    );
                    SetValue { value }
                }
                (SetValue { value }, DeleteCandidates { candidates })
                | (DeleteCandidates { candidates }, SetValue { value }) => {
                    ensure!(
                        !candidates.has(value),
                        "can't set deleted candidate {value} as a value"
                    );
                    SetValue { value }
                }
                (DeleteCandidate { candidate }, DeleteCandidates { candidates })
                | (DeleteCandidates { candidates }, DeleteCandidate { candidate }) => {
                    DeleteCandidates {
                        candidates: candidates.union(Candidates::single(candidate)),
                    }
                }
            })
        })()
        .with_context(|| format!("Incompatible merge of two actions: {self}, {other}"))?;

        Ok(())
    }
}

pub mod transport {
    use super::*;

    #[cfg_attr(
        feature = "wasm",
        derive(TS),
        ts(export, export_generic_params = "crate::base::consts::Base3")
    )]
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
    #[serde(bound = "", rename_all = "camelCase")]
    pub struct TransportDeduction<Base: SudokuBase> {
        pub reasons: Vec<TransportReason<Base>>,
        pub actions: Vec<TransportAction<Base>>,
    }

    impl<Base: SudokuBase> From<Deduction<Base>> for TransportDeduction<Base> {
        fn from(deduction: Deduction<Base>) -> Self {
            Self {
                reasons: deduction
                    .reasons
                    .iter()
                    .map(|(position, reason)| TransportReason {
                        position,
                        reason: *reason,
                    })
                    .collect(),
                actions: deduction
                    .actions
                    .iter()
                    .map(|(position, action)| TransportAction {
                        position,
                        action: *action,
                    })
                    .collect(),
            }
        }
    }

    #[cfg_attr(
        feature = "wasm",
        derive(TS),
        ts(export, export_generic_params = "crate::base::consts::Base3")
    )]
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
    #[serde(bound = "", rename_all = "camelCase")]
    pub struct TransportReason<Base: SudokuBase> {
        pub position: Position<Base>,
        #[serde(flatten)]
        pub reason: Reason<Base>,
    }
    #[cfg_attr(
        feature = "wasm",
        derive(TS),
        ts(export, export_generic_params = "crate::base::consts::Base3")
    )]
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
    #[serde(bound = "", rename_all = "camelCase")]
    pub struct TransportAction<Base: SudokuBase> {
        pub position: Position<Base>,
        #[serde(flatten)]
        pub action: Action<Base>,
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;
    use crate::solver::strategic::deduction::transport::{TransportAction, TransportReason};

    use super::*;

    #[test]
    fn test_new_deduction() {
        use crate::solver::strategic::deduction::transport::TransportDeduction;
        use serde_json;

        let deduction = TransportDeduction::<Base3> {
            reasons: vec![
                TransportReason {
                    position: (1, 1).try_into().unwrap(),
                    reason: Reason::Candidates {
                        candidates: Candidates::all(),
                    },
                },
                TransportReason {
                    position: (1, 1).try_into().unwrap(),
                    reason: Reason::Candidate {
                        candidate: Value::try_from(1).unwrap(),
                    },
                },
            ],
            actions: vec![
                TransportAction {
                    position: (1, 1).try_into().unwrap(),
                    action: Action::SetValue {
                        value: Value::try_from(1).unwrap(),
                    },
                },
                TransportAction {
                    position: (1, 1).try_into().unwrap(),
                    action: Action::DeleteCandidate {
                        candidate: Value::try_from(2).unwrap(),
                    },
                },
                TransportAction {
                    position: (1, 1).try_into().unwrap(),
                    action: Action::DeleteCandidates {
                        candidates: Candidates::all(),
                    },
                },
            ],
        };

        println!("{}", serde_json::to_string_pretty(&deduction).unwrap());
    }

    // TODO: port test to new Deductions

    // #[test]
    // fn test_deductions_order_independence() {
    //     use itertools::Itertools;
    //
    //     let pos = DynamicPosition { row: 0, column: 0 };
    //     let previous_candidates: Candidates<Base2> = Candidates::all();
    //     let remaining_candidates: Candidates<Base2> = vec![1, 2].try_into().unwrap();
    //
    //     let value_deduction_1 =
    //         OldDeduction::with_value(pos, previous_candidates, 1.try_into().unwrap()).unwrap();
    //     let value_deduction_2 = OldDeduction::with_value(
    //         DynamicPosition { row: 1, column: 1 },
    //         previous_candidates,
    //         2.try_into().unwrap(),
    //     )
    //     .unwrap();
    //     let remaining_candidates_deduction =
    //         OldDeduction::with_remaining_candidates(pos, previous_candidates, remaining_candidates)
    //             .unwrap();
    //
    //     let all_deductions: Vec<OldDeduction<Base2>> = vec![
    //         value_deduction_1,
    //         value_deduction_2,
    //         remaining_candidates_deduction,
    //     ];
    //     let deductions: OldDeductions<Base2> =
    //         IntoDeductions(all_deductions.clone()).try_into().unwrap();
    //     for deduction_permutation in all_deductions.into_iter().permutations(3) {
    //         assert_eq!(
    //             OldDeductions::<Base2>::try_from(IntoDeductions(deduction_permutation)).unwrap(),
    //             deductions
    //         );
    //     }
    // }
    //
    // #[test]
    // fn test_deduction_apply() {
    //     let mut grid = samples::base_2_candidates_coordinates();
    //
    //     let pos = DynamicPosition { row: 0, column: 1 };
    //     let value = 1.try_into().unwrap();
    //     OldDeduction::with_value(pos, Candidates::single(1.try_into().unwrap()), value)
    //         .unwrap()
    //         .apply(&mut grid);
    //     assert_eq!(*grid.get(pos), Cell::with_value(value, false));
    //
    //     let pos = DynamicPosition { row: 3, column: 3 };
    //     let candidates = vec![2, 4].try_into().unwrap();
    //     OldDeduction::with_remaining_candidates(pos, Candidates::all(), candidates)
    //         .unwrap()
    //         .apply(&mut grid);
    //     assert_eq!(*grid.get(pos), Cell::with_candidates(candidates));
    // }
    //
    // #[test]
    // fn test_deduction_merge() {
    //     let pos = DynamicPosition { row: 1, column: 1 };
    //     let previous_candidates: Candidates<Base2> = Candidates::all();
    //     let remaining_candidates: Candidates<Base2> = Candidates::single(1.try_into().unwrap());
    //     let value: Value<Base2> = 1.try_into().unwrap();
    //
    //     let cases: Vec<(
    //         OldDeduction<Base2>,
    //         OldDeduction<Base2>,
    //         OldDeduction<Base2>,
    //     )> = vec![
    //         // Equal
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //         ),
    //         // Left Value overwrites right PruneCandidates
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //         ),
    //         // Right Value overwrites left PruneCandidates
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //         ),
    //         // Intersect PruneCandidates
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 vec![1, 2, 4].try_into().unwrap(),
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 vec![1, 3, 4].try_into().unwrap(),
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 vec![1, 4].try_into().unwrap(),
    //             )
    //             .unwrap(),
    //         ),
    //     ];
    //
    //     for (left_strategy, right_strategy, expected_strategy) in cases {
    //         assert_eq!(
    //             left_strategy.merge(&right_strategy).unwrap(),
    //             expected_strategy
    //         );
    //     }
    // }
    // #[test]
    // fn test_deduction_merge_err() {
    //     let pos = DynamicPosition { row: 1, column: 1 };
    //     let different_pos = DynamicPosition { row: 2, column: 2 };
    //     let previous_candidates: Candidates<Base2> = Candidates::all();
    //     let different_previous_candidates: Candidates<Base2> = vec![1, 2].try_into().unwrap();
    //     let remaining_candidates: Candidates<Base2> = Candidates::single(1.try_into().unwrap());
    //     let different_remaining_candidates: Candidates<Base2> =
    //         Candidates::single(2.try_into().unwrap());
    //     let value: Value<Base2> = 1.try_into().unwrap();
    //     let different_value: Value<Base2> = 2.try_into().unwrap();
    //
    //     let err_cases: Vec<(OldDeduction<Base2>, OldDeduction<Base2>)> = vec![
    //         // Different pos
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(different_pos, previous_candidates, value).unwrap(),
    //         ),
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 different_pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //         ),
    //         // Different previous_candidates
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, different_previous_candidates, value).unwrap(),
    //         ),
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 different_previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //         ),
    //         // Different value
    //         (
    //             OldDeduction::with_value(pos, previous_candidates, value).unwrap(),
    //             OldDeduction::with_value(pos, previous_candidates, different_value).unwrap(),
    //         ),
    //         // No intersection
    //         (
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 remaining_candidates,
    //             )
    //             .unwrap(),
    //             OldDeduction::with_remaining_candidates(
    //                 pos,
    //                 previous_candidates,
    //                 different_remaining_candidates,
    //             )
    //             .unwrap(),
    //         ),
    //     ];
    //
    //     for (left_strategy, right_strategy) in err_cases {
    //         assert!(left_strategy.merge(&right_strategy).is_err());
    //     }
    // }
}
