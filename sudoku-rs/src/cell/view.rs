use std::convert::TryFrom;
use std::convert::TryInto;

use crate::base::consts::BaseMax;
use anyhow::bail;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::cell_state::CellState;
use crate::cell::Cell;
use crate::error::{Error, Result};

pub(crate) mod parser;

// TODO: unify representation for empty cell
//  Cell: Empty Candidates
//  CellView: Unfixed value
//  => Constructor now validates this, but Deserialize can break this contract

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum CellView {
    Value { value: u8, fixed: bool },
    Candidates { candidates: Vec<u8> },
}

impl CellView {
    pub fn value(value: u8, fixed: bool) -> Self {
        if value == 0 {
            if fixed {
                panic!("An empty cell can't be fixed")
            } else {
                Self::candidates(vec![])
            }
        } else {
            CellView::Value { value, fixed }
        }
    }

    pub fn candidates(candidates: Vec<u8>) -> Self {
        CellView::Candidates { candidates }
    }
}

pub fn v(value: u8) -> CellView {
    CellView::value(value, false)
}

pub fn f(value: u8) -> CellView {
    CellView::value(value, true)
}

pub fn c(candidates: Vec<u8>) -> CellView {
    CellView::candidates(candidates)
}

fn char_value_to_u8(c: char) -> Result<u8> {
    match c {
        '.' | '0' => Ok(0),
        _ => match c.to_digit(36) {
            Some(digit) => Ok(digit.try_into()?),
            None => bail!("Unable to convert character into number: {}", c),
        },
    }
}

impl TryFrom<&str> for CellView {
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

impl TryFrom<char> for CellView {
    type Error = Error;

    fn try_from(c: char) -> Result<Self> {
        Ok(char_value_to_u8(c)?.into())
    }
}

impl TryFrom<u32> for CellView {
    type Error = Error;

    fn try_from(bits: u32) -> Result<Self> {
        let candidates = Candidates::<BaseMax>::with_integral(bits);
        let candidates_vec = candidates.to_vec_u8();

        Ok(if let &[value] = candidates_vec.as_slice() {
            Self::value(value, false)
        } else {
            Self::candidates(candidates_vec)
        })
    }
}

impl From<u8> for CellView {
    fn from(value: u8) -> Self {
        CellView::value(value, false)
    }
}

impl<Base: SudokuBase> From<&Cell<Base>> for CellView {
    fn from(cell: &Cell<Base>) -> Self {
        match cell.state() {
            CellState::Value(value) => CellView::Value {
                value: value.into_u8(),
                fixed: false,
            },
            CellState::FixedValue(value) => CellView::Value {
                value: value.into_u8(),
                fixed: true,
            },
            CellState::Candidates(candidates) => CellView::Candidates {
                candidates: candidates.to_vec_u8(),
            },
        }
    }
}
