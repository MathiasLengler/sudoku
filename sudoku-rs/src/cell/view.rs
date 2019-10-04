use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum CellView {
    Value { value: usize },
    Candidates { candidates: Vec<usize> },
}

impl CellView {
    pub fn v(value: usize) -> Self {
        CellView::Value { value }
    }

    pub fn c(candidates: Vec<usize>) -> Self {
        CellView::Candidates { candidates }
    }
}

// TODO: into SudokuCell with max

// TODO: From<SudokuCell>
// TODO: From<usize>
// TODO: From<str> (candidates)
