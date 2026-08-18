use crate::{
    base::SudokuBase,
    error::{Error, Result},
    position::{DynamicPosition, Position},
};

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

// FIXME: replace with macro rules

// Mirrors `TryFrom<_> for Position` impls in sudoku-rs/src/position/bounded_position.rs.
// We can't define this generically, since it would conflict with the `From<(Position<Base>, T)> for Positioned<Base, T>` impl above.

impl<Base: SudokuBase, T, IntoT: TryInto<T, Error: Into<Error>>> TryFrom<((u8, u8), IntoT)>
    for Positioned<Base, T>
{
    type Error = Error;

    fn try_from((into_pos, into_t): ((u8, u8), IntoT)) -> Result<Self> {
        Ok(Self {
            pos: into_pos.try_into()?,
            value: into_t.try_into().map_err(Into::into)?,
        })
    }
}

impl<Base: SudokuBase, T, IntoT: TryInto<T, Error: Into<Error>>> TryFrom<(u16, IntoT)>
    for Positioned<Base, T>
{
    type Error = Error;

    fn try_from((into_pos, into_t): (u16, IntoT)) -> Result<Self> {
        Ok(Self {
            pos: into_pos.try_into()?,
            value: into_t.try_into().map_err(Into::into)?,
        })
    }
}

impl<Base: SudokuBase, T, IntoT: TryInto<T, Error: Into<Error>>> TryFrom<(DynamicPosition, IntoT)>
    for Positioned<Base, T>
{
    type Error = Error;

    fn try_from((into_pos, into_t): (DynamicPosition, IntoT)) -> Result<Self> {
        Ok(Self {
            pos: into_pos.try_into()?,
            value: into_t.try_into().map_err(Into::into)?,
        })
    }
}
