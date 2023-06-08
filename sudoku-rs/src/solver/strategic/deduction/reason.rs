use std::fmt::{Display, Formatter};

use anyhow::{bail, ensure, Context};

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Cell;
use crate::cell::CellState;
use crate::cell::Value;
use crate::error::{Error, Result};
use crate::position::Merge;
use crate::solver::strategic::deduction::transport::TransportReason;

/// On what basis a deduction was made.
/// Used to highlight/explain a deduction in the UI.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Reason<Base: SudokuBase> {
    /// This set of candidates was a reason for an action.
    /// Highlighted with a green background in the UI.
    Candidates(Candidates<Base>),
    // TODO: add Reason::Cell for group highlighting
}

impl<Base: SudokuBase> TryFrom<TransportReason> for Reason<Base> {
    type Error = Error;

    fn try_from(transport_reason: TransportReason) -> Result<Self> {
        let TransportReason::Candidates(candidates) = transport_reason;
        Ok(Self::Candidates(candidates.try_into()?))
    }
}

impl<Base: SudokuBase> Display for Reason<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::Candidates(candidates) => {
                if let Some(candidate) = candidates.to_single() {
                    write!(f, "candidate {candidate}")
                } else {
                    write!(f, "candidates {candidates}")
                }
            }
        }
    }
}

impl<Base: SudokuBase> Reason<Base> {
    pub fn candidate(candidate: Value<Base>) -> Self {
        Self::Candidates(Candidates::with_single(candidate))
    }

    pub fn candidates(candidates: Candidates<Base>) -> Self {
        Self::Candidates(candidates)
    }

    pub fn validate(&self, cell: &Cell<Base>) -> Result<()> {
        (|| {
            match cell.state() {
                CellState::Value(value) | CellState::FixedValue(value) => {
                    bail!("unexpected cell with value {value}")
                }
                CellState::Candidates(existing_candidates) => match *self {
                    Reason::Candidates(candidates) => {
                        ensure!(!candidates.is_empty(), "candidates must not be empty");
                        let unexpected_candidates = candidates.without(*existing_candidates);
                        ensure!(
                            unexpected_candidates.is_empty(),
                            "unexpected candidates {unexpected_candidates}"
                        );
                    }
                },
            }
            Ok(())
        })()
        .with_context(|| format!("Invalid reason {self} for cell {cell}"))
    }
}

impl<Base: SudokuBase> Merge for Reason<Base> {
    fn merge(self, other: Self) -> Result<Self> {
        Ok(match (self, other) {
            (Reason::Candidates(candidates), Reason::Candidates(other_candidates)) => {
                Reason::Candidates(candidates.union(other_candidates))
            }
        })
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
        let candidates_1 = Candidates::with_single(value_1);
        let candidates_2 = Candidates::with_single(value_2);
        let candidates_1_2: Candidates<_> = [value_1, value_2].into_iter().collect();

        let test_cases = vec![
            (candidates_1, candidates_1, candidates_1),
            (candidates_2, candidates_2, candidates_2),
            (candidates_1_2, candidates_1_2, candidates_1_2),
        ];

        for (reason_1, reason_2, expected_reason) in
            test_cases
                .into_iter()
                .map(|(candidates_1, candidates_2, expected_candidates)| {
                    (
                        Reason::Candidates(candidates_1),
                        Reason::Candidates(candidates_2),
                        Reason::Candidates(expected_candidates),
                    )
                })
        {
            let merged_reason_1_2 = reason_1.merge(reason_2).unwrap();
            let merged_reason_2_1 = reason_2.merge(reason_1).unwrap();
            assert_eq!(merged_reason_1_2, expected_reason);
            assert_eq!(merged_reason_2_1, expected_reason);
        }
    }
}
