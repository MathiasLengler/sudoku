use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU8;

use anyhow::{ensure, format_err};
use serde::{Serialize, Serializer};

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicValue;
use crate::error::{Error, Result};
use crate::position::Coordinate;

/// A valid sudoku value for a given base.
///
/// A `Value` always is in the range of `1..=(Base::MAX_VALUE)`
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub struct Value<Base: SudokuBase> {
    value: NonZeroU8,
    base: PhantomData<Base>,
}

impl<Base: SudokuBase> Default for Value<Base> {
    fn default() -> Self {
        1.try_into().unwrap()
    }
}

/// Constructors
impl<Base: SudokuBase> Value<Base> {
    /// Ok(Some(value)) => value in legal range
    /// Ok(None) => 0
    /// Err(err) => value too big
    pub fn new(value: u8) -> Result<Option<Self>> {
        Self::validate_value(value, false)?;

        Ok(NonZeroU8::new(value).map(|value| Self {
            value,
            base: PhantomData,
        }))
    }

    /// # Safety
    ///
    /// `value` must be in the range `1..=(Base::MAX_VALUE)`.
    pub unsafe fn new_unchecked(value: u8) -> Self {
        Self::debug_assert_value(value, true);

        Self {
            // Safety: value is-none zero, upheld by the caller safety contract.
            value: unsafe { NonZeroU8::new_unchecked(value) },
            base: PhantomData,
        }
    }

    pub fn max() -> Self {
        // Safety: `Base::MAX_VALUE` is in the range `1..=(Base::MAX_VALUE)`
        unsafe { Self::new_unchecked(Base::MAX_VALUE) }
    }
}

/// Validation
impl<Base: SudokuBase> Value<Base> {
    fn validate_value(value: u8, ensure_non_zero: bool) -> Result<()> {
        let max_value = Base::MAX_VALUE;
        ensure!(
            value <= max_value,
            "Value can't be greater than {max_value}, instead got: {value}"
        );
        if ensure_non_zero {
            ensure!(value != 0, "Value can't be 0");
        }
        Ok(())
    }

    fn assert_value(value: u8, ensure_non_zero: bool) {
        Self::validate_value(value, ensure_non_zero).unwrap();
    }

    fn debug_assert_value(value: u8, ensure_non_zero: bool) {
        debug_assert!({
            Self::assert_value(value, ensure_non_zero);
            true
        });
    }
}

/// Getters
impl<Base: SudokuBase> Value<Base> {
    /// Get the `value` as a `u8`.
    /// Guaranteed to be in the range `1..=(Base::MAX_VALUE)`.
    pub fn get(self) -> u8 {
        self.value.get()
    }
}

/// Iterators
impl<Base: SudokuBase> Value<Base> {
    pub fn all() -> impl Iterator<Item = Self> {
        (1..=Base::MAX_VALUE).map(|value|
            // Safety: `value` remains in-bounds
            unsafe { Self::new_unchecked(value) })
    }
}

impl<Base: SudokuBase> Display for Value<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use radix_fmt::radix_32;

        radix_32(self.value).fmt(f)
    }
}

impl<Base: SudokuBase> From<Coordinate<Base>> for Value<Base> {
    fn from(coordinate: Coordinate<Base>) -> Self {
        let coordinate = coordinate.get();
        // Safety:
        // `Coordinate::<Base::get` guarantees `coordinate < Base::SIDE_LENGTH`.
        // `SudokuBase` guarantees `Base::SIDE_LENGTH == Base::MAX_VALUE`.
        // Therefore `coordinate + 1` is inside the range `1..=(Base::MAX_VALUE)`.
        unsafe { Self::new_unchecked(coordinate + 1) }
    }
}

impl<Base: SudokuBase> TryFrom<u8> for Value<Base> {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::new(value)?.ok_or_else(|| format_err!("Value can't be 0"))
    }
}

impl<Base: SudokuBase> TryFrom<DynamicValue> for Value<Base> {
    type Error = Error;

    fn try_from(dynamic_value: DynamicValue) -> Result<Self> {
        dynamic_value.0.try_into()
    }
}

impl<Base: SudokuBase> Serialize for Value<Base> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.value.get())
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    #[test]
    fn test_new() -> Result<()> {
        let value = Value::<Base3>::new(0)?;
        assert!(value.is_none());

        let value = Value::<Base3>::new(9)?;
        assert_eq!(value.map(|value| value.get()), Some(9));

        let value = Value::<Base3>::new(10);
        assert!(value.is_err());

        Ok(())
    }

    #[test]
    fn test_try_from() -> Result<()> {
        let value = Value::<Base3>::try_from(0);
        assert!(value.is_err());

        let value = Value::<Base3>::try_from(9)?;
        assert_eq!(value.get(), 9);

        let value = Value::<Base3>::try_from(10);
        assert!(value.is_err());

        Ok(())
    }
}
