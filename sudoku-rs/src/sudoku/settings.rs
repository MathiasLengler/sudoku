use crate::sudoku::history::DEFAULT_LIMIT;
use serde::{Deserialize, Serialize};

// TODO: add public settings API
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug, Copy, Serialize, Deserialize)]
pub struct Settings {
    pub update_candidates: bool,
    pub history_limit: usize,
    pub find_solution: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            update_candidates: true,
            history_limit: DEFAULT_LIMIT,
            find_solution: true,
        }
    }
}
