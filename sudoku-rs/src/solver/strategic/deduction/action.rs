use std::fmt::{Display, Formatter};

use anyhow::{bail, ensure, Context};

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Cell;
use crate::cell::Value;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Merge;
use crate::position::Position;
use crate::solver::strategic::deduction::transport::TransportAction;

/// What action should be taken for a specific cell.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Action<Base: SudokuBase> {
    SetValue(Value<Base>),
    DeleteCandidates(Candidates<Base>),
}

impl<Base: SudokuBase> TryFrom<TransportAction> for Action<Base> {
    type Error = Error;

    fn try_from(transport_action: TransportAction) -> Result<Self> {
        Ok(match transport_action {
            TransportAction::SetValue(value) => Self::SetValue(value.try_into()?),
            TransportAction::DeleteCandidates(candidates) => {
                Self::DeleteCandidates(candidates.try_into()?)
            }
        })
    }
}

impl<Base: SudokuBase> Display for Action<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::SetValue(value) => {
                write!(f, "set value {value}")
            }
            Action::DeleteCandidates(candidates) => {
                if let Some(candidate) = candidates.to_single() {
                    write!(f, "delete candidate {candidate}")
                } else {
                    write!(f, "delete candidates {candidates}")
                }
            }
        }
    }
}

impl<Base: SudokuBase> Action<Base> {
    pub fn set_value(value: Value<Base>) -> Self {
        Self::SetValue(value)
    }

    pub fn delete_candidate(candidate: Value<Base>) -> Self {
        Self::DeleteCandidates(Candidates::with_single(candidate))
    }

    pub fn delete_candidates(candidates: Candidates<Base>) -> Self {
        Self::DeleteCandidates(candidates)
    }

    pub fn validate(&self, cell: &Cell<Base>) -> Result<Candidates<Base>> {
        (|| {
            let Some(existing_candidates) = cell.candidates() else {
            bail!("expected cell to contain candidates")
        };
            match *self {
                Action::SetValue(value) => {
                    ensure!(
                        existing_candidates.has(value),
                        "expected cell to contain the candidate {value}"
                    );
                }
                Action::DeleteCandidates(candidates) => {
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

    pub fn apply(&self, cell: &mut Cell<Base>) -> Result<()> {
        let existing_candidates = self.validate(cell)?;
        match *self {
            Action::SetValue(value) => {
                cell.set_value(value);
                // Defer updating of direct candidates
            }
            Action::DeleteCandidates(candidates) => {
                if let Some(candidate) = candidates.to_single() {
                    cell.delete_candidate(candidate);
                } else {
                    cell.set_candidates(existing_candidates.without(candidates));
                }
            }
        }

        Ok(())
    }

    pub fn update_direct_candidates(&self, grid: &mut Grid<Base>, pos: Position<Base>) {
        if let Action::SetValue(value) = self {
            grid.update_direct_candidates(pos, *value);
        }
    }
}

impl<Base: SudokuBase> Merge for Action<Base> {
    fn merge(self, other: Self) -> Result<Self> {
        use Action::*;
        (|| {
            Ok(match (self, other) {
                (SetValue(self_value), SetValue(other_value)) => {
                    ensure!(
                        self_value == other_value,
                        "conflicting values: {self_value} != {other_value}"
                    );
                    SetValue(self_value)
                }
                (DeleteCandidates(self_candidates), DeleteCandidates(other_candidates)) => {
                    DeleteCandidates(self_candidates.union(other_candidates))
                }
                (SetValue(value), DeleteCandidates(candidates))
                | (DeleteCandidates(candidates), SetValue(value)) => {
                    ensure!(
                        !candidates.has(value),
                        "can't set deleted candidate {value} as a value"
                    );
                    SetValue(value)
                }
            })
        })()
        .with_context(|| format!("Incompatible merge of two actions: {self}, {other}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;

    use super::*;

    #[test]
    fn test_merge() {
        type Base = Base2;
        let value_1: Value<Base> = 1.try_into().unwrap();
        let value_2: Value<Base> = 2.try_into().unwrap();

        let set_1 = Action::SetValue(value_1);
        let set_2 = Action::SetValue(value_2);
        let delete_1 = Action::DeleteCandidates(Candidates::with_single(value_1));
        let delete_2 = Action::DeleteCandidates(Candidates::with_single(value_2));
        let delete_1_2 = Action::DeleteCandidates([value_1, value_2].into_iter().collect());
        let delete_none = Action::DeleteCandidates(Candidates::<Base>::new());

        let test_cases_ok = vec![
            (set_1, set_1, set_1),
            (set_2, set_2, set_2),
            (delete_1, delete_1, delete_1),
            (delete_2, delete_2, delete_2),
            (delete_1_2, delete_1_2, delete_1_2),
            (delete_none, delete_none, delete_none),
            (set_1, delete_2, set_1),
            (set_1, delete_none, set_1),
            (set_2, delete_1, set_2),
            (set_2, delete_none, set_2),
            (delete_1, delete_2, delete_1_2),
            (delete_1, delete_1_2, delete_1_2),
            (delete_1, delete_none, delete_1),
            (delete_2, delete_1_2, delete_1_2),
            (delete_2, delete_none, delete_2),
            (delete_1_2, delete_none, delete_1_2),
        ];
        let test_cases_err = vec![
            (set_1, set_2),
            (set_1, delete_1),
            (set_1, delete_1_2),
            (set_2, delete_2),
            (set_2, delete_1_2),
        ];

        for (action_1, action_2, expected_action) in test_cases_ok {
            let merged_action_1_2 = action_1.merge(action_2).unwrap();
            let merged_action_2_1 = action_2.merge(action_1).unwrap();
            assert_eq!(merged_action_1_2, expected_action);
            assert_eq!(merged_action_2_1, expected_action);
        }
        for (action_1, action_2) in test_cases_err {
            action_1.merge(action_2).unwrap_err();
            action_2.merge(action_1).unwrap_err();
        }
    }
}
