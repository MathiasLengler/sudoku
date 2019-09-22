use std::num::NonZeroUsize;
use std::usize;

// TODO: add public settings API
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct Settings {
    pub update_candidates_on_set_value: bool,
    pub history_limit: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            update_candidates_on_set_value: true,
            history_limit: usize::MAX,
        }
    }
}
