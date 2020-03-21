use std::usize;

// TODO: add public settings API
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug, Copy)]
pub struct Settings {
    pub update_candidates: bool,
    pub history_limit: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            update_candidates: true,
            history_limit: 256,
        }
    }
}
