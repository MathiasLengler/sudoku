use std::fmt::{self, Display};
use std::ops::{Div, Mul};

use serde::{Deserialize, Serialize};

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

impl Position {
    pub fn index_tuple(&self) -> (usize, usize) {
        let &Position { row, column } = self;
        (row.into(), column.into())
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
