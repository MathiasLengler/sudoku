use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::grid::index::coordinate::BaseCoordinate;
use crate::position::Position;
use anyhow::ensure;
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct BasePosition<Base: SudokuBase> {
    /// # Safety invariants
    /// - `cell_index < Base::CELL_COUNT`
    cell_index: u16,
    _base: PhantomData<Base>,
}

/// Constructors
impl<Base: SudokuBase> BasePosition<Base> {
    pub fn new(cell_index: u16) -> Result<Self> {
        Self::validate_cell_index(cell_index)?;
        // Safety: we have validated `cell_index` above.
        let this = unsafe { Self::new_unchecked(cell_index) };
        Ok(this)
    }

    /// # Safety
    ///
    /// `cell_index < Base::CELL_COUNT` must be true.
    pub(crate) unsafe fn new_unchecked(cell_index: u16) -> Self {
        let this = Self {
            cell_index,
            _base: Default::default(),
        };
        this.debug_assert();
        this
    }
}

/// Validation
impl<Base: SudokuBase> BasePosition<Base> {
    fn validate_cell_index(cell_index: u16) -> Result<()> {
        ensure!(cell_index < Base::CELL_COUNT);
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        Self::validate_cell_index(self.cell_index)
    }

    fn assert(&self) {
        self.validate().unwrap()
    }

    pub(crate) fn debug_assert(&self) {
        debug_assert!({
            self.assert();
            true
        });
    }
}

/// Getters
impl<Base: SudokuBase> BasePosition<Base> {
    pub fn cell_index(&self) -> u16 {
        self.cell_index
    }

    pub fn row(&self) -> BaseCoordinate<Base> {
        let row = self.cell_index / u16::from(Base::SIDE_LENGTH);

        // Safety: the calculation for `row` always remains in-bounds.
        unsafe { BaseCoordinate::new_unchecked_u16(row) }
    }

    pub fn column(&self) -> BaseCoordinate<Base> {
        let column = self.cell_index % u16::from(Base::SIDE_LENGTH);

        // Safety: the calculation for `column` always remains in-bounds.
        unsafe { BaseCoordinate::new_unchecked_u16(column) }
    }

    pub fn row_and_column(&self) -> (BaseCoordinate<Base>, BaseCoordinate<Base>) {
        (self.row(), self.column())
    }
}

impl<Base: SudokuBase> From<(BaseCoordinate<Base>, BaseCoordinate<Base>)> for BasePosition<Base> {
    fn from((row, column): (BaseCoordinate<Base>, BaseCoordinate<Base>)) -> Self {
        row.debug_assert();
        column.debug_assert();

        let cell_index =
            u16::from(row.get()) * u16::from(Base::SIDE_LENGTH) + u16::from(column.get());

        // Safety: the calculation for `cell_index` always remains in-bounds,
        // since `row` and `column` are each bounds checked at creation-time.
        unsafe { Self::new_unchecked(cell_index) }
    }
}

impl<Base: SudokuBase> TryFrom<(u8, u8)> for BasePosition<Base> {
    type Error = Error;

    fn try_from((row, column): (u8, u8)) -> Result<Self> {
        let row = BaseCoordinate::<Base>::try_from(row)?;
        let column = BaseCoordinate::<Base>::try_from(column)?;
        Ok((row, column).into())
    }
}

impl<Base: SudokuBase> TryFrom<u16> for BasePosition<Base> {
    type Error = Error;

    fn try_from(cell_index: u16) -> Result<Self> {
        Self::new(cell_index)
    }
}

impl<Base: SudokuBase> TryFrom<Position> for BasePosition<Base> {
    type Error = Error;

    fn try_from(Position { row, column }: Position) -> Result<Self> {
        (row, column).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::{Base2, Base3};

    #[test]
    fn test_new() {
        // Base 2
        assert_eq!(BasePosition::<Base2>::new(0).unwrap().cell_index, 0);
        assert_eq!(BasePosition::<Base2>::new(15).unwrap().cell_index, 15);
        assert!(BasePosition::<Base2>::new(16).is_err());

        // Base 3
        assert_eq!(BasePosition::<Base3>::new(0).unwrap().cell_index, 0);
        assert_eq!(BasePosition::<Base3>::new(80).unwrap().cell_index, 80);
        assert!(BasePosition::<Base3>::new(81).is_err());
    }
}
