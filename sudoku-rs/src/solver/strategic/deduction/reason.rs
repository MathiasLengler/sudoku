use std::fmt::{Display, Formatter};

use anyhow::{bail, ensure, Context};
use serde::Serialize;
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::cell_state::CellState;
use crate::cell::compact::value::Value;
use crate::cell::Cell;
use crate::error::Result;
use crate::solver::strategic::deduction::Merge;

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
    pub fn validate(&self, cell: &Cell<Base>) -> Result<()> {
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
    fn merge(self, other: Self) -> Result<Self> {
        Ok(match (self, other) {
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
        })
    }
}
