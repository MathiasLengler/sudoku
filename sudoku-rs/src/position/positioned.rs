use crate::{base::SudokuBase, position::Position};

/// Some `value` at a cell position `pos`
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Positioned<Base: SudokuBase, T> {
    pub pos: Position<Base>,
    pub value: T,
}

impl<Base: SudokuBase, T> From<(Position<Base>, T)> for Positioned<Base, T> {
    fn from((pos, value): (Position<Base>, T)) -> Self {
        Self { pos, value }
    }
}

impl<Base: SudokuBase, T> From<Positioned<Base, T>> for (Position<Base>, T) {
    fn from(Positioned { pos, value }: Positioned<Base, T>) -> Self {
        (pos, value)
    }
}
