use std::fmt::{self, Display};
use std::ops::{Div, Mul};

// TODO: Marker for Cell/Block
// TODO: make fields private and add getters
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Pos").field(&self.x).field(&self.y).finish()
    }
}

impl Div for Position {
    type Output = Position;

    fn div(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Div<usize> for Position {
    type Output = Position;

    fn div(self, rhs: usize) -> Self::Output {
        self / Position {
            x: rhs,
            y: rhs,
        }
    }
}

impl Mul for Position {
    type Output = Position;

    fn mul(self, rhs: Position) -> Self::Output {
        Position {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Mul<usize> for Position {
    type Output = Position;

    fn mul(self, rhs: usize) -> Self::Output {
        self * Position {
            x: rhs,
            y: rhs,
        }
    }
}
