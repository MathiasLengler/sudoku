use std::fmt::{self, Display, Formatter, Debug};
use std::num::NonZeroUsize;

pub trait SudokuCell: Default + Clone + Display + Debug + Ord + Eq + Send {
    fn has_value(&self) -> bool;
    fn new_with_value(value: usize) -> Self;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct OptionCell(pub Option<NonZeroUsize>);

impl Display for OptionCell {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(&match self.0 {
            None => "_".to_string(),
            Some(value) => value.to_string(),
        })
    }
}

impl SudokuCell for OptionCell {
    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn new_with_value(value: usize) -> Self {
        match NonZeroUsize::new(value) {
            Some(value) => OptionCell(Some(value)),
            None => OptionCell(None),
        }
    }
}