use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use std::num::NonZeroU8;

use anyhow::{ensure, format_err};
use serde::export::Formatter;
use typenum::Unsigned;

use crate::base::SudokuBase;
use crate::error::{Error, Result};

/// A valid sudoku value for a given base.
///
/// A `Value` always is in the range of `1..=(base^2)`
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct Value<Base: SudokuBase> {
    value: NonZeroU8,
    base: PhantomData<Base>,
}

impl<Base: SudokuBase> Value<Base> {
    /// Ok(Some(value)) => value in legal range
    /// Ok(None) => 0
    /// Err(err) => value too big
    pub fn new(value: u8) -> Result<Option<Self>> {
        let limit = Base::MaxValue::to_u8();

        ensure!(value <= limit, "Value can't be greater than {}", limit);

        Ok(NonZeroU8::new(value).map(|value| Self {
            value,
            base: Default::default(),
        }))
    }

    pub fn into_u8(self) -> u8 {
        self.value.get()
    }
}

impl<Base: SudokuBase> TryFrom<u8> for Value<Base> {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::new(value)?.ok_or_else(|| format_err!("Value can't be 0"))
    }
}

impl<Base: SudokuBase> Display for Value<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
