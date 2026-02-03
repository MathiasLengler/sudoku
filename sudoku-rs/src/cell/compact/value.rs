use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU8;

use anyhow::{ensure, format_err};
use serde::{Deserialize, Serialize};

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicValue;
use crate::error::{Error, Result};
use crate::position::Coordinate;

pub mod map {
    use super::*;
    use crate::grid::group::Group;
    use std::fmt::Debug;

    // TODO: implement `Grid::count_values() -> ValueMap<Base, u16>` (CELL_COUNT: u16)
    // then implement ValueMap<Base, u16>::(variance/deviation/dispersion)
    //  decide on which dispersion to implement: https://en.wikipedia.org/wiki/Statistical_dispersion

    /// A map of `Value<Base>` to `T`.
    #[derive(Debug, Default, Clone)]
    pub struct ValueMap<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug> {
        values: Group<Base, T>,
    }

    impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + PartialEq> PartialEq
        for ValueMap<Base, T>
    where
        Base::Group<T>: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.values == other.values
        }
    }

    impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Eq> Eq for ValueMap<Base, T> where
        Base::Group<T>: Eq
    {
    }

    impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug> ValueMap<Base, T> {
        pub fn with_all(value: T) -> Self
        where
            T: Copy,
        {
            Self {
                values: Group::with_all(value),
            }
        }

        pub fn get(&self, value: Value<Base>) -> T {
            self.values.get(value.into())
        }
        pub fn get_mut(&mut self, value: Value<Base>) -> &mut T {
            self.values.get_mut(value.into())
        }
    }

    pub type ValueCounts<Base> = ValueMap<Base, u16>;

    impl<Base: SudokuBase> ValueCounts<Base> {
        pub fn count(values: impl IntoIterator<Item = Value<Base>>) -> Self {
            let mut this = Self::default();
            for value in values {
                *this.get_mut(value) += 1;
            }
            this
        }

        pub fn sum(&self) -> u16 {
            self.values.iter().sum()
        }

        pub fn std_dev(&self) -> f64 {
            use statrs::statistics::Statistics;

            self.values.iter().map(f64::from).std_dev()
        }
    }
}

/// A valid sudoku value for a given base.
///
/// A `Value` always is in the range of `1..=(Base::MAX_VALUE)`
#[allow(
    clippy::unsafe_derive_deserialize,
    reason = "Safety invariants upheld by serde(try_from)"
)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub struct Value<Base: SudokuBase> {
    /// # Safety invariants
    /// - `value <= Base::MAX_VALUE`
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
    /// - `Ok(Some(value))` => value in legal range
    /// - `Ok(None)` => 0
    /// - `Err(err)` => value too big
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

    pub fn middle() -> Self {
        let middle_value = Base::MAX_VALUE.div_ceil(2);
        // Safety: `middle_value` is in the range `1..=(Base::MAX_VALUE)`
        unsafe { Self::new_unchecked(middle_value) }
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
    pub fn all() -> impl DoubleEndedIterator<Item = Self> + Clone {
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

impl<Base: SudokuBase> From<Value<Base>> for u8 {
    fn from(value: Value<Base>) -> Self {
        value.get()
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
        value.unwrap_err();

        Ok(())
    }

    #[test]
    fn test_try_from() -> Result<()> {
        let value = Value::<Base3>::try_from(0);
        value.unwrap_err();

        let value = Value::<Base3>::try_from(9)?;
        assert_eq!(value.get(), 9);

        let value = Value::<Base3>::try_from(10);
        value.unwrap_err();

        Ok(())
    }

    #[test]
    fn test_middle() {
        assert_eq!(Value::<Base2>::middle().get(), 2);
        assert_eq!(Value::<Base3>::middle().get(), 5);
        assert_eq!(Value::<Base4>::middle().get(), 8);
        assert_eq!(Value::<Base5>::middle().get(), 13);
    }

    mod serde {
        use super::*;
        use crate::test_util::test_all_bases;
        use serde_test::{Token, assert_tokens};

        mod default {
            use super::*;
            test_all_bases!({
                assert_tokens(&Value::<Base>::default(), &[Token::U8(1)]);
            });
        }

        mod max {
            use super::*;
            test_all_bases!({
                let value = Value::<Base>::default();
                assert_tokens(&value, &[Token::U8(value.get())]);
            });
        }

        mod middle {
            use super::*;
            test_all_bases!({
                let value = Value::<Base>::middle();
                assert_tokens(&value, &[Token::U8(value.get())]);
            });
        }
    }
}
