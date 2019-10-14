use std::convert::TryInto;

use failure::_core::convert::TryFrom;
use failure::bail;
use serde::{Deserialize, Serialize};

use crate::cell::SudokuCell;
use crate::error::{Error, Result};

#[derive(Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum CellView {
    Value { value: usize },
    Candidates { candidates: Vec<usize> },
}

impl CellView {
    pub fn value(value: usize) -> Self {
        CellView::Value { value }
    }

    pub fn candidates(candidates: Vec<usize>) -> Self {
        CellView::Candidates { candidates }
    }

    pub fn is_value(&self) -> bool {
        match self {
            CellView::Value { .. } => true,
            CellView::Candidates { .. } => false,
        }
    }

    pub fn into_cell<Cell: SudokuCell>(self, max: usize) -> Cell {
        match self {
            CellView::Value { value } => Cell::new_with_value(value, max),
            CellView::Candidates { candidates } => Cell::new_with_candidates(candidates, max),
        }
    }
}

pub fn v(value: usize) -> CellView {
    CellView::value(value)
}

pub fn c(candidates: Vec<usize>) -> CellView {
    CellView::candidates(candidates)
}

fn char_value_to_usize(c: char) -> Result<usize> {
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
                        let candidate = char_value_to_usize(candidate)?;
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
        Ok(char_value_to_usize(c)?.into())
    }
}

impl From<usize> for CellView {
    fn from(value: usize) -> Self {
        CellView::value(value)
    }
}

impl<Cell: SudokuCell> From<Cell> for CellView {
    fn from(cell: Cell) -> Self {
        cell.view()
    }
}
