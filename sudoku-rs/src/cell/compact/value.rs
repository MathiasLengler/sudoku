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
    pub fn into_u8(self) -> u8 {
        self.value.get()
    }
}

impl<Base: SudokuBase> TryFrom<u8> for Value<Base> {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        let limit = Base::MaxValue::to_u8();

        ensure!(value <= limit, "Value can't be greater than {}", limit);

        let value = NonZeroU8::new(value).ok_or_else(|| format_err!("Value can't be 0"))?;

        Ok(Self {
            value,
            base: Default::default(),
        })
    }
}

impl<Base: SudokuBase> Display for Value<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
