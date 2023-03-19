use std::marker::PhantomData;

use anyhow::ensure;

use crate::base::SudokuBase;
use crate::error::{Error, Result};

// TODO: index wrappers for:
//  Row(BaseCoordinate)
//  Column(BaseCoordinate)
//  Block(BaseCoordinate)

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct BaseCoordinate<Base: SudokuBase> {
    /// # Safety invariants
    /// - `coordinate < Base::SIDE_LENGTH`
    coordinate: u8,
    _base: PhantomData<Base>,
}

/// Constructors
impl<Base: SudokuBase> BaseCoordinate<Base> {
    pub fn new(coordinate: u8) -> Result<Self> {
        Self::validate_coordinate(coordinate)?;
        // Safety: we have validated `coordinate` above.
        let this = unsafe { Self::new_unchecked(coordinate) };
        Ok(this)
    }

    /// # Safety
    ///
    /// `coordinate < Base::SIDE_LENGTH` must be true.
    pub(super) unsafe fn new_unchecked(coordinate: u8) -> Self {
        let this = Self {
            coordinate,
            _base: Default::default(),
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

        let coordinate = coordinate as u8;

        Self::new_unchecked(coordinate)
    }
}

/// Validation
impl<Base: SudokuBase> BaseCoordinate<Base> {
    fn validate_coordinate(coordinate: u8) -> Result<()> {
        ensure!(coordinate < Base::SIDE_LENGTH);
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        Self::validate_coordinate(self.coordinate)
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
impl<Base: SudokuBase> BaseCoordinate<Base> {
    pub fn get(&self) -> u8 {
        self.coordinate
    }
}

impl<Base: SudokuBase> TryFrom<u8> for BaseCoordinate<Base> {
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
        assert_eq!(BaseCoordinate::<Base2>::new(0).unwrap().coordinate, 0);
        assert_eq!(BaseCoordinate::<Base2>::new(3).unwrap().coordinate, 3);
        assert!(BaseCoordinate::<Base2>::new(4).is_err());

        // Base 3
        assert_eq!(BaseCoordinate::<Base3>::new(0).unwrap().coordinate, 0);
        assert_eq!(BaseCoordinate::<Base3>::new(8).unwrap().coordinate, 8);
        assert!(BaseCoordinate::<Base3>::new(9).is_err());
    }
}
