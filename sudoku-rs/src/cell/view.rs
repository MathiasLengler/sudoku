use std::convert::TryInto;

use failure::_core::convert::TryFrom;
use failure::bail;
use serde::{Deserialize, Serialize};

use crate::cell::{Cell, SudokuBase};
use crate::error::{Error, Result};

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum CellView {
    Value { value: u8, fixed: bool },
    Candidates { candidates: Vec<u8> },
}

impl CellView {
    pub fn value(value: u8, fixed: bool) -> Self {
        CellView::Value { value, fixed }
    }

    pub fn candidates(candidates: Vec<u8>) -> Self {
        CellView::Candidates { candidates }
    }

    pub fn is_value(&self) -> bool {
        match self {
            CellView::Value { .. } => true,
            CellView::Candidates { .. } => false,
        }
    }

    pub fn into_cell<Base: SudokuBase>(self) -> Cell<Base> {
        match self {
            CellView::Value { value, fixed } => Cell::with_value(value, fixed),
            CellView::Candidates { candidates } => Cell::with_candidates(candidates),
        }
    }
}

pub fn v(value: u8) -> CellView {
    CellView::value(value, false)
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

impl From<u8> for CellView {
    fn from(value: u8) -> Self {
        CellView::value(value, false)
    }
}

impl<Base: SudokuBase> From<Cell<Base>> for CellView {
    fn from(cell: Cell<Base>) -> Self {
        cell.view()
    }
}
