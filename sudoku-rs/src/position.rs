use std::fmt::{self, Display};
use std::ops::{Div, Mul};

use serde::{Deserialize, Serialize};

// TODO: Marker for Cell/Block
//  compare with euclid (except x, y bad for clarity)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Pos")
            .field(&self.row)
            .field(&self.column)
            .finish()
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

impl Div<usize> for Position {
    type Output = Position;

    fn div(self, rhs: usize) -> Self::Output {
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

impl Mul<usize> for Position {
    type Output = Position;

    fn mul(self, rhs: usize) -> Self::Output {
        self * Position {
            row: rhs,
            column: rhs,
        }
    }
}
