use std::fmt::{self, Display, Formatter};

pub trait SudokuCell: Default + Clone + Display + Ord + Eq {
    fn has_value(&self) -> bool;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Debug)]
pub struct OptionCell(pub Option<u8>);

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
}