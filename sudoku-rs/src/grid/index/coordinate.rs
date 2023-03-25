use std::marker::PhantomData;

use anyhow::ensure;

use crate::base::SudokuBase;
use crate::error::{Error, Result};

/// A coordinate/index in a sudoku grid.
///
/// Can represent three different dimensions:
///
/// # Row
/// # Column
/// # Block
/// TODO: visualize
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Coordinate<Base: SudokuBase> {
    /// # Safety invariants
    /// - `coordinate < Base::SIDE_LENGTH`
    coordinate: u8,
    _base: PhantomData<Base>,
}

/// Constructors
impl<Base: SudokuBase> Coordinate<Base> {
    pub fn new(coordinate: u8) -> Result<Self> {
        Self::validate_coordinate(coordinate)?;
        // Safety: we have validated `coordinate` above.
        let this = unsafe { Self::new_unchecked(coordinate) };
        Ok(this)
    }

    pub fn max() -> Self {
        let max_coordinate = Base::SIDE_LENGTH - 1;

        // Safety: `Base::SIDE_LENGTH` is always non-zero, so `max_coordinate` remains in-bounds.
        unsafe { Self::new_unchecked(max_coordinate) }
    }

    /// # Safety
    ///
    /// `coordinate < Base::SIDE_LENGTH` must be true.
    pub(crate) unsafe fn new_unchecked(coordinate: u8) -> Self {
        let this = Self {
            coordinate,
            _base: PhantomData,
        };
        this.debug_assert();
        this
    }

    /// # Safety
    ///
    /// `coordinate < Base::SIDE_LENGTH` must be true.
    pub(crate) unsafe fn new_unchecked_u16(coordinate: u16) -> Self {
        // Test for truncation.
        debug_assert!({
            u8::try_from(coordinate).unwrap();
            true
        });

        #[allow(clippy::cast_possible_truncation)]
        let coordinate = coordinate as u8;

        Self::new_unchecked(coordinate)
    }
}

/// Validation
impl<Base: SudokuBase> Coordinate<Base> {
    fn validate_coordinate(coordinate: u8) -> Result<()> {
        ensure!(coordinate < Base::SIDE_LENGTH);
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        Self::validate_coordinate(self.coordinate)
    }

    fn assert(&self) {
        self.validate().unwrap();
    }

    pub(crate) fn debug_assert(&self) {
        debug_assert!({
            self.assert();
            true
        });
    }
}

/// Getters
impl<Base: SudokuBase> Coordinate<Base> {
    pub fn get(self) -> u8 {
        self.coordinate
    }

    pub fn get_u16(self) -> u16 {
        u16::from(self.coordinate)
    }
}

/// Iterators
impl<Base: SudokuBase> Coordinate<Base> {
    pub fn all() -> impl Iterator<Item = Self> {
        (0..Base::SIDE_LENGTH).map(|coordinate|
            // Safety: `coordinate` remains in-bounds
            unsafe { Self::new_unchecked(coordinate) })
    }
}

impl<Base: SudokuBase> TryFrom<u8> for Coordinate<Base> {
    type Error = Error;

    fn try_from(coordinate: u8) -> Result<Self> {
        Self::new(coordinate)
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::{Base2, Base3};

    use super::*;

    #[test]
    fn test_new() {
        // Base 2
        assert_eq!(Coordinate::<Base2>::new(0).unwrap().coordinate, 0);
        assert_eq!(Coordinate::<Base2>::new(3).unwrap().coordinate, 3);
        assert!(Coordinate::<Base2>::new(4).is_err());
    }

    #[test]
    fn test_all() {
        use itertools::assert_equal;

        assert_equal(
            Coordinate::<Base3>::all(),
            (0..9).map(|coordinate| Coordinate::new(coordinate).unwrap()),
        );
    }

    #[test]
    fn test_max() {
        assert_eq!(Coordinate::<Base2>::max().get(), 3);
        assert_eq!(Coordinate::<Base3>::max().get(), 8);
    }
}
