// TODO: add public settings API
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct Settings {
    pub update_candidates_on_set_value: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            update_candidates_on_set_value: true,
        }
    }
}
