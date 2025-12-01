use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::base::consts::BaseMax;
use crate::base::SudokuBase;
use crate::cell::{Candidates, Cell, CellState, Value};
use crate::error::{Error, Result};

// TODO: unify representation for empty cell
//  Cell: Empty Candidates
//  CellView: Unfixed value
//  => Constructor now validates this, but Deserialize can break this contract

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct DynamicValue(pub u8);

impl From<u8> for DynamicValue {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl<Base: SudokuBase> From<Value<Base>> for DynamicValue {
    fn from(value: Value<Base>) -> Self {
        Self(value.get())
    }
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct DynamicCandidates(pub Vec<u8>);

impl From<Vec<u8>> for DynamicCandidates {
    fn from(candidates: Vec<u8>) -> Self {
        // TODO: change to try_from and check for 0
        Self(candidates)
    }
}

impl<Base: SudokuBase> From<Candidates<Base>> for DynamicCandidates {
    fn from(candidates: Candidates<Base>) -> Self {
        Self(candidates.to_vec_u8())
    }
}

// FIXME: `tag = "kind"` leads to larger serialized size
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum DynamicCell {
    Value { value: DynamicValue, fixed: bool },
    Candidates { candidates: DynamicCandidates },
}

impl DynamicCell {
    pub fn value(value: u8, fixed: bool) -> Self {
        if value == 0 {
            if fixed {
                panic!("An empty cell can't be fixed")
            } else {
                Self::candidates(vec![])
            }
        } else {
            Self::Value {
                value: value.into(),
                fixed,
            }
        }
    }

    pub fn candidates(candidates: Vec<u8>) -> Self {
        Self::Candidates {
            candidates: candidates.into(),
        }
    }
}

pub fn v(value: u8) -> DynamicCell {
    DynamicCell::value(value, false)
}

pub fn f(value: u8) -> DynamicCell {
    DynamicCell::value(value, true)
}

pub fn c(candidates: Vec<u8>) -> DynamicCell {
    DynamicCell::candidates(candidates)
}

pub(crate) fn char_value_to_u8(c: char) -> Result<u8> {
    match c {
        '.' | '0' => Ok(0),
        _ => match c.to_digit(36) {
            Some(digit) => Ok(digit.try_into()?),
            None => bail!("Unable to convert character into number: {}", c),
        },
    }
}

impl TryFrom<&str> for DynamicCell {
    type Error = Error;

    fn try_from(candidates: &str) -> Result<Self> {
        match candidates.len() {
            0 => bail!("Unexpected empty string while parsing candidates"),
            1 => candidates.chars().next().unwrap().try_into(),
            _ => Ok(Self::candidates(
                candidates
                    .chars()
                    .map(|candidate| {
                        let candidate = char_value_to_u8(candidate)?;
                        if candidate == 0 {
                            bail!("A candidate can't be 0")
                        } else {
                            Ok(candidate)
                        }
                    })
                    .collect::<Result<Vec<_>>>()?,
            )),
        }
    }
}

impl TryFrom<char> for DynamicCell {
    type Error = Error;

    fn try_from(c: char) -> Result<Self> {
        Ok(char_value_to_u8(c)?.into())
    }
}

impl TryFrom<u32> for DynamicCell {
    type Error = Error;

    fn try_from(bits: u32) -> Result<Self> {
        let candidates = Candidates::<BaseMax>::with_integral(bits)?;
        let candidates_vec = candidates.to_vec_u8();

        Ok(if let &[value] = candidates_vec.as_slice() {
            Self::value(value, false)
        } else {
            Self::candidates(candidates_vec)
        })
    }
}

impl From<u8> for DynamicCell {
    fn from(value: u8) -> Self {
        DynamicCell::value(value, false)
    }
}

impl From<DynamicCandidates> for DynamicCell {
    fn from(candidates: DynamicCandidates) -> Self {
        Self::Candidates { candidates }
    }
}

impl<Base: SudokuBase> From<&Cell<Base>> for DynamicCell {
    fn from(cell: &Cell<Base>) -> Self {
        cell.clone().into()
    }
}

impl<Base: SudokuBase> From<Cell<Base>> for DynamicCell {
    fn from(cell: Cell<Base>) -> Self {
        match *cell.state() {
            CellState::Value(value) => Self::Value {
                value: value.into(),
                fixed: false,
            },
            CellState::FixedValue(value) => Self::Value {
                value: value.into(),
                fixed: true,
            },
            CellState::Candidates(candidates) => Self::Candidates {
                candidates: candidates.into(),
            },
        }
    }
}
