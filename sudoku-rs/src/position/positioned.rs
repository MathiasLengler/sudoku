use crate::{base::SudokuBase, position::Position};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Positioned<Base: SudokuBase, T> {
    pub pos: Position<Base>,
    pub item: T,
}

impl<Base: SudokuBase, T> From<(Position<Base>, T)> for Positioned<Base, T> {
    fn from((pos, item): (Position<Base>, T)) -> Self {
        Self { pos, item }
    }
}

impl<Base: SudokuBase, T> From<Positioned<Base, T>> for (Position<Base>, T) {
    fn from(Positioned { pos, item }: Positioned<Base, T>) -> Self {
        (pos, item)
    }
}
