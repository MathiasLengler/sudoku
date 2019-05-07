use std::fmt::{self, Display};
use std::ops::{Div, Mul};

use serde::{Deserialize, Serialize};

// TODO: Marker for Cell/Block
//  use euclid
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Position {
    pub column: usize,
    pub row: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Pos")
            .field(&self.column)
            .field(&self.row)
            .finish()
    }
}

impl Div for Position {
    type Output = Position;

    fn div(self, rhs: Self) -> Self::Output {
        Position {
            column: self.column / rhs.column,
            row: self.row / rhs.row,
        }
    }
}

impl Div<usize> for Position {
    type Output = Position;

    fn div(self, rhs: usize) -> Self::Output {
        self / Position {
            column: rhs,
            row: rhs,
        }
    }
}

impl Mul for Position {
    type Output = Position;

    fn mul(self, rhs: Position) -> Self::Output {
        Position {
            column: self.column * rhs.column,
            row: self.row * rhs.row,
        }
    }
}

impl Mul<usize> for Position {
    type Output = Position;

    fn mul(self, rhs: usize) -> Self::Output {
        self * Position {
            column: rhs,
            row: rhs,
        }
    }
}
