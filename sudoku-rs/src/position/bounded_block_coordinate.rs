use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use anyhow::ensure;

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::position::Coordinate;

/// A coordinate in a sudoku block.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct BlockCoordinate<Base: SudokuBase> {
    /// # Safety invariants
    /// - `block_coordinate < Base::BASE`
    block_coordinate: u8,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase> Display for BlockCoordinate<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.block_coordinate)
    }
}

/// Constructors
impl<Base: SudokuBase> BlockCoordinate<Base> {
    pub fn new(block_coordinate: u8) -> Result<Self> {
        Self::validate_block_coordinate(block_coordinate)?;
        // Safety: we have validated `block_coordinate` above.
        let this = unsafe { Self::new_unchecked(block_coordinate) };
        Ok(this)
    }

    pub fn max() -> Self {
        let max_block_coordinate = Base::BASE - 1;

        // Safety: `Base::BASE` is always non-zero, so `max_block_coordinate` remains in-bounds.
        unsafe { Self::new_unchecked(max_block_coordinate) }
    }

    pub fn round_down(coordinate: Coordinate<Base>) -> Self {
        // Safety: relies on invariants:
        // - Coordinate<Base>::get: `coordinate < Base::SIDE_LENGTH`
        // - `SudokuBase`: `Base::SIDE_LENGTH` equals `BASE.pow(2)`
        // Therefore `(coordinate.get() / Base::BASE) < Base::BASE` holds.
        unsafe { Self::new_unchecked(coordinate.get() / Base::BASE) }
    }

    /// # Safety
    ///
    /// `coordinate < Base::BASE` must be true.
    pub(crate) unsafe fn new_unchecked(coordinate: u8) -> Self {
        let this = Self {
            block_coordinate: coordinate,
            _base: PhantomData,
        };
        this.debug_assert();
        this
    }
}

/// Validation
impl<Base: SudokuBase> BlockCoordinate<Base> {
    fn validate_block_coordinate(block_coordinate: u8) -> Result<()> {
        ensure!(block_coordinate < Base::BASE);
        Ok(())
    }

    fn validate(self) -> Result<()> {
        Self::validate_block_coordinate(self.block_coordinate)
    }

    fn assert(self) {
        self.validate().unwrap();
    }

    pub(crate) fn debug_assert(self) {
        debug_assert!({
            self.assert();
            true
        });
    }
}

/// Getters
impl<Base: SudokuBase> BlockCoordinate<Base> {
    /// Get the `block_coordinate` as a `u8`.
    /// Guaranteed to satisfy `coordinate < Base::BASE`
    pub fn get(self) -> u8 {
        self.block_coordinate
    }

    /// Get the `block_coordinate` as a `u16`.
    /// Guaranteed to satisfy `coordinate < Base::BASE`
    pub fn get_u16(self) -> u16 {
        u16::from(self.block_coordinate)
    }

    /// Get the `block_coordinate` as a `usize`.
    /// Guaranteed to satisfy `coordinate < Base::BASE`
    pub fn get_usize(self) -> usize {
        usize::from(self.block_coordinate)
    }
}

/// Iterators
impl<Base: SudokuBase> BlockCoordinate<Base> {
    pub fn all() -> impl Iterator<Item = Self> {
        (0..Base::BASE).map(|coordinate|
            // Safety: `block_coordinate` remains in-bounds
            unsafe { Self::new_unchecked(coordinate) })
    }
}

impl<Base: SudokuBase> TryFrom<u8> for BlockCoordinate<Base> {
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
        assert_eq!(
            BlockCoordinate::<Base2>::new(0).unwrap().block_coordinate,
            0
        );
        assert_eq!(
            BlockCoordinate::<Base2>::new(1).unwrap().block_coordinate,
            1
        );
        BlockCoordinate::<Base2>::new(2).unwrap_err();
    }

    #[test]
    fn test_all() {
        use itertools::assert_equal;

        assert_equal(
            BlockCoordinate::<Base3>::all(),
            (0..3).map(|coordinate| BlockCoordinate::new(coordinate).unwrap()),
        );
    }

    #[test]
    fn test_max() {
        assert_eq!(BlockCoordinate::<Base2>::max().get(), 1);
        assert_eq!(BlockCoordinate::<Base3>::max().get(), 2);
    }
}
