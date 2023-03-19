use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU8;

use anyhow::{ensure, format_err};
use serde::{Serialize, Serializer};

use crate::base::SudokuBase;
use crate::error::{Error, Result};

/// A valid sudoku value for a given base.
///
/// A `Value` always is in the range of `1..=(Base::MAX_VALUE)`
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub struct Value<Base: SudokuBase> {
    value: NonZeroU8,
    base: PhantomData<Base>,
}

impl<Base: SudokuBase> Value<Base> {
    /// Ok(Some(value)) => value in legal range
    /// Ok(None) => 0
    /// Err(err) => value too big
    pub fn new(value: u8) -> Result<Option<Self>> {
        let limit = Base::MAX_VALUE;

        ensure!(
            value <= limit,
            "Value can't be greater than {limit}, instead got: {value}"
        );

        Ok(NonZeroU8::new(value).map(|value| Self {
            value,
            base: PhantomData::default(),
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

impl<Base: SudokuBase> Serialize for Value<Base> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.value.get())
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use ts_rs::TS;

    use super::*;

    impl<Base: SudokuBase> TS for Value<Base> {
        const EXPORT_TO: Option<&'static str> = Some("bindings/Value.ts");
        fn decl() -> String {
            "type Value = number;".to_owned()
        }
        fn name() -> String {
            "Value".to_owned()
        }
        fn name_with_type_args(_args: Vec<String>) -> String {
            Self::name()
        }
        fn inline() -> String {
            "number".to_owned()
        }
        fn dependencies() -> Vec<ts_rs::Dependency> {
            vec![]
        }
        fn transparent() -> bool {
            false
        }
    }

    #[cfg(test)]
    #[test]
    fn export_bindings_value() {
        use crate::base::consts::Base3;

        <Value<Base3> as ts_rs::TS>::export().expect("could not export type");
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    #[test]
    fn test_new() -> Result<()> {
        let value = Value::<U3>::new(0)?;
        assert!(value.is_none());

        let value = Value::<U3>::new(9)?;
        assert_eq!(value.map(|value| value.into_u8()), Some(9));

        let value = Value::<U3>::new(10);
        assert!(value.is_err());

        Ok(())
    }

    #[test]
    fn test_try_from() -> Result<()> {
        let value = Value::<U3>::try_from(0);
        assert!(value.is_err());

        let value = Value::<U3>::try_from(9)?;
        assert_eq!(value.into_u8(), 9);

        let value = Value::<U3>::try_from(10);
        assert!(value.is_err());

        Ok(())
    }
}
