use serde::Serialize;
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::grid::index::position::Position;
use crate::solver::strategic::deduction::{Action, Deduction, Reason};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::Base3;
    use crate::cell::compact::candidates::Candidates;
    use crate::cell::compact::value::Value;

    #[test]
    fn test_debug_serde() {
        use serde_json;

        let deduction = TransportDeduction::<Base3> {
            reasons: vec![
                TransportReason {
                    position: (1, 1).try_into().unwrap(),
                    reason: Reason::Candidates(Candidates::all()),
                },
                TransportReason {
                    position: (1, 1).try_into().unwrap(),
                    reason: Reason::candidate(Value::try_from(1).unwrap()),
                },
            ],
            actions: vec![
                TransportAction {
                    position: (1, 1).try_into().unwrap(),
                    action: Action::set_value(Value::try_from(1).unwrap()),
                },
                TransportAction {
                    position: (1, 1).try_into().unwrap(),
                    action: Action::delete_candidate(Value::try_from(2).unwrap()),
                },
                TransportAction {
                    position: (1, 1).try_into().unwrap(),
                    action: Action::delete_candidates(Candidates::all()),
                },
            ],
        };

        println!("{}", serde_json::to_string_pretty(&deduction).unwrap());
    }
}
