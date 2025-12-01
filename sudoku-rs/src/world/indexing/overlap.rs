use crate::base::SudokuBase;
use crate::error::{Error, Result};
use anyhow::ensure;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// How many cells grids overlap each other in the world.
///
/// In principle, this could be any number in the range `0..Base::SIDE_LENGTH`.
/// However, a overlap greater than `Base::BASE` would:
/// - result in overly constrained or trivial puzzles
/// - complicate the implementation of the world
///
/// Therefore, we restrict the overlap to the range `0..=Base::BASE`.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Serialize, Deserialize,
)]
#[serde(into = "u8", try_from = "u8")]
pub struct GridOverlap<Base: SudokuBase> {
    /// # Safety invariants
    /// - `overlap <= Base::BASE`
    overlap: u8,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase> Display for GridOverlap<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.overlap)
    }
}

/// Constructors
impl<Base: SudokuBase> GridOverlap<Base> {
    pub fn new(overlap: u8) -> Result<Self> {
        Self::validate_overlap(overlap)?;
        // Safety: we have validated `overlap` above.
        let this = unsafe { Self::new_unchecked(overlap) };
        Ok(this)
    }

    pub fn max() -> Self {
        let max_overlap = Base::BASE;

        // Safety: `Base::BASE` is always non-zero, so `max_overlap` remains in-bounds.
        unsafe { Self::new_unchecked(max_overlap) }
    }

    /// # Safety
    ///
    /// `overlap <= Base::BASE` must be true.
    pub(crate) unsafe fn new_unchecked(overlap: u8) -> Self {
        let this = Self {
            overlap,
            _base: PhantomData,
        };
        this.debug_assert();
        this
    }
}

/// Validation
impl<Base: SudokuBase> GridOverlap<Base> {
    fn validate_overlap(overlap: u8) -> Result<()> {
        ensure!(overlap <= Base::BASE);
        Ok(())
    }

    fn validate(self) -> Result<()> {
        Self::validate_overlap(self.overlap)
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
impl<Base: SudokuBase> GridOverlap<Base> {
    /// Get the `overlap` as a `u8`.
    /// Guaranteed to satisfy `overlap <= Base::BASE`
    pub fn get(self) -> u8 {
        self.overlap
    }

    /// Get the `overlap` as a `u16`.
    /// Guaranteed to satisfy `overlap <= Base::BASE`
    pub fn get_u16(self) -> u16 {
        u16::from(self.overlap)
    }

    /// Get the `overlap` as a `usize`.
    /// Guaranteed to satisfy `overlap <= Base::BASE`
    pub fn get_usize(self) -> usize {
        usize::from(self.overlap)
    }
    /// Get the `overlap` as a `isize`.
    /// Guaranteed to be contained in the range `0..=Base::BASE`
    pub fn get_isize(self) -> isize {
        isize::from(self.overlap)
    }

    /// The cell distance between the start of grids in the world.
    pub fn grid_stride(self) -> u8 {
        let grid_stride = Base::SIDE_LENGTH - self.overlap;
        debug_assert!(grid_stride > 0, "grid_stride must be positive");
        grid_stride
    }

    pub fn grid_stride_usize(self) -> usize {
        usize::from(self.grid_stride())
    }
}

/// Iterators
impl<Base: SudokuBase> GridOverlap<Base> {
    pub fn all() -> impl Iterator<Item = Self> {
        (0..=Base::BASE).map(|overlap|
            // Safety: `overlap` remains in-bounds
            unsafe { Self::new_unchecked(overlap) })
    }
    pub fn all_non_zero() -> impl Iterator<Item = Self> {
        (1..=Base::BASE).map(|overlap|
            // Safety: `overlap` remains in-bounds
            unsafe { Self::new_unchecked(overlap) })
    }
}

impl<Base: SudokuBase> TryFrom<u8> for GridOverlap<Base> {
    type Error = Error;

    fn try_from(overlap: u8) -> Result<Self> {
        Self::new(overlap)
    }
}

impl<Base: SudokuBase> From<GridOverlap<Base>> for u8 {
    fn from(value: GridOverlap<Base>) -> Self {
        value.get()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::{Base2, Base3};

    use super::*;

    #[test]
    fn test_new() {
        // Base 2
        assert_eq!(GridOverlap::<Base2>::new(0).unwrap().overlap, 0);
        assert_eq!(GridOverlap::<Base2>::new(1).unwrap().overlap, 1);
        assert_eq!(GridOverlap::<Base2>::new(2).unwrap().overlap, 2);
        GridOverlap::<Base2>::new(3).unwrap_err();
    }

    #[test]
    fn test_all() {
        use itertools::assert_equal;

        assert_equal(
            GridOverlap::<Base3>::all(),
            (0..=3).map(|overlap| GridOverlap::new(overlap).unwrap()),
        );
    }

    #[test]
    fn test_all_non_zero() {
        use itertools::assert_equal;

        assert_equal(
            GridOverlap::<Base3>::all_non_zero(),
            (1..=3).map(|overlap| GridOverlap::new(overlap).unwrap()),
        );
    }

    #[test]
    fn test_max() {
        assert_eq!(GridOverlap::<Base2>::max().get(), 2);
        assert_eq!(GridOverlap::<Base3>::max().get(), 3);
    }
}
