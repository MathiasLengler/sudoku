use std::fmt::{self, Display};
use std::ops::{Div, Mul};

use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::grid::index::position::BasePosition;

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub struct Position {
    pub row: u8,
    pub column: u8,
}

impl From<(u8, u8)> for Position {
    fn from((row, column): (u8, u8)) -> Self {
        Self { row, column }
    }
}

impl<Base: SudokuBase> From<BasePosition<Base>> for Position {
    fn from(base_position: BasePosition<Base>) -> Self {
        let (row, column) = base_position.row_and_column();
        (row.get(), column.get()).into()
    }
}
impl Position {
    pub fn index_tuple(&self) -> (usize, usize) {
        let &Position { row, column } = self;
        (row.into(), column.into())
    }

    pub fn cell_index<Base: SudokuBase>(&self) -> u16 {
        u16::from(self.row) * u16::from(Base::SIDE_LENGTH) + u16::from(self.column)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}c{}", self.row, self.column)
    }
}

impl Div for Position {
    type Output = Position;

    fn div(self, rhs: Self) -> Self::Output {
        Position {
            row: self.row / rhs.row,
            column: self.column / rhs.column,
        }
    }
}

impl Div<u8> for Position {
    type Output = Position;

    fn div(self, rhs: u8) -> Self::Output {
        self / Position {
            row: rhs,
            column: rhs,
        }
    }
}

impl Mul for Position {
    type Output = Position;

    fn mul(self, rhs: Position) -> Self::Output {
        Position {
            row: self.row * rhs.row,
            column: self.column * rhs.column,
        }
    }
}

impl Mul<u8> for Position {
    type Output = Position;

    fn mul(self, rhs: u8) -> Self::Output {
        self * Position {
            row: rhs,
            column: rhs,
        }
    }
}
