use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::dynamic::{DynamicCandidates, DynamicValue};
use crate::position::DynamicPosition;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransportDeductions {
    deductions: Vec<TransportDeduction>,
}

impl<Base: SudokuBase> From<Deductions<Base>> for TransportDeductions {
    fn from(deductions: Deductions<Base>) -> Self {
        Self {
            deductions: deductions.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransportDeduction {
    pub reasons: Vec<PositionedTransportReason>,
    pub actions: Vec<PositionedTransportAction>,
}

impl<Base: SudokuBase> From<Deduction<Base>> for TransportDeduction {
    fn from(deduction: Deduction<Base>) -> Self {
        Self {
            reasons: deduction
                .reasons
                .into_iter()
                .map(|(position, reason)| PositionedTransportReason {
                    position: position.into(),
                    reason: reason.into(),
                })
                .collect(),
            actions: deduction
                .actions
                .into_iter()
                .map(|(position, action)| PositionedTransportAction {
                    position: position.into(),
                    action: action.into(),
                })
                .collect(),
        }
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PositionedTransportReason {
    pub position: DynamicPosition,
    #[serde(flatten)]
    pub reason: TransportReason,
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransportReason {
    Candidates(DynamicCandidates),
}

impl<Base: SudokuBase> From<Reason<Base>> for TransportReason {
    fn from(reason: Reason<Base>) -> Self {
        let Reason::Candidates(candidates) = reason;
        Self::Candidates(candidates.into())
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PositionedTransportAction {
    pub position: DynamicPosition,
    #[serde(flatten)]
    pub action: TransportAction,
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransportAction {
    SetValue(DynamicValue),
    DeleteCandidates(DynamicCandidates),
}

impl<Base: SudokuBase> From<Action<Base>> for TransportAction {
    fn from(action: Action<Base>) -> Self {
        match action {
            Action::SetValue(value) => Self::SetValue(value.into()),
            Action::DeleteCandidates(candidates) => Self::DeleteCandidates(candidates.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: test round-trip

    #[test]
    fn test_debug_serde() {
        use serde_json;

        let deduction = TransportDeduction {
            reasons: vec![
                PositionedTransportReason {
                    position: (1, 2).into(),
                    reason: TransportReason::Candidates(vec![1, 2, 3].into()),
                },
                PositionedTransportReason {
                    position: (3, 4).into(),
                    reason: TransportReason::Candidates(vec![4].into()),
                },
            ],
            actions: vec![
                PositionedTransportAction {
                    position: (1, 1).try_into().unwrap(),
                    action: TransportAction::SetValue(1.into()),
                },
                PositionedTransportAction {
                    position: (2, 2).try_into().unwrap(),
                    action: TransportAction::DeleteCandidates(vec![1].into()),
                },
                PositionedTransportAction {
                    position: (3, 3).try_into().unwrap(),
                    action: TransportAction::DeleteCandidates(vec![1, 2, 3].into()),
                },
            ],
        };

        println!("{}", serde_json::to_string_pretty(&deduction).unwrap());
    }
}
