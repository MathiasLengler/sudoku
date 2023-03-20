use std::fmt::{self, Display};
use std::ops::{Div, Mul};

use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::grid::index::position::Position;

/// The position of a cell in a grid of unknown size.
#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub struct DynamicPosition {
    pub row: u8,
    pub column: u8,
}

impl From<(u8, u8)> for DynamicPosition {
    fn from((row, column): (u8, u8)) -> Self {
        Self { row, column }
    }
}

impl<Base: SudokuBase> From<Position<Base>> for DynamicPosition {
    fn from(base_position: Position<Base>) -> Self {
        let (row, column) = base_position.to_row_and_column();
        (row.get(), column.get()).into()
    }
}
impl DynamicPosition {
    pub fn index_tuple(&self) -> (usize, usize) {
        let &DynamicPosition { row, column } = self;
        (row.into(), column.into())
    }

    pub fn cell_index<Base: SudokuBase>(&self) -> u16 {
        u16::from(self.row) * u16::from(Base::SIDE_LENGTH) + u16::from(self.column)
    }
}

impl Display for DynamicPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}c{}", self.row, self.column)
    }
}

impl Div for DynamicPosition {
    type Output = DynamicPosition;

    fn div(self, rhs: Self) -> Self::Output {
        DynamicPosition {
            row: self.row / rhs.row,
            column: self.column / rhs.column,
        }
    }
}

impl Div<u8> for DynamicPosition {
    type Output = DynamicPosition;

    fn div(self, rhs: u8) -> Self::Output {
        self / DynamicPosition {
            row: rhs,
            column: rhs,
        }
    }
}

impl Mul for DynamicPosition {
    type Output = DynamicPosition;

    fn mul(self, rhs: DynamicPosition) -> Self::Output {
        DynamicPosition {
            row: self.row * rhs.row,
            column: self.column * rhs.column,
        }
    }
}

impl Mul<u8> for DynamicPosition {
    type Output = DynamicPosition;

    fn mul(self, rhs: u8) -> Self::Output {
        self * DynamicPosition {
            row: rhs,
            column: rhs,
        }
    }
}
