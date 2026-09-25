use crate::{
    error::Result,
    world::{RelativeDir, WorldDim, WorldObject},
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

use super::ValidatedWorldPosition;

// FIMXE: fix TS export
//  this type only makes sense if it is "branded" by the generic
//  we could continue to refer to zod branded schemas or
//  implement that somehow ourself

/// A position of a `WorldObject`.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export), ts(concrete(T = crate::world::CellMarker)))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct WorldPosition<T: WorldObject> {
    pub row: u32,
    pub column: u32,
    #[cfg_attr(feature = "wasm", ts(skip))]
    #[serde(skip)]
    object: PhantomData<T>,
}

impl<T: WorldObject> Display for WorldPosition<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { row, column, .. } = self;
        write!(f, "({row}, {column})")
    }
}

impl<T: WorldObject> WorldPosition<T> {
    pub fn new(row: u32, column: u32) -> Self {
        Self {
            row,
            column,
            object: PhantomData,
        }
    }

    pub fn contained_in(self, grid_dim: WorldDim<T>) -> bool {
        grid_dim.contains(self)
    }

    pub(in crate::world) fn validate(self, dim: WorldDim<T>) -> Result<ValidatedWorldPosition<T>> {
        ValidatedWorldPosition::new(self, dim)
    }

    pub fn is_at_top_edge(self) -> bool {
        self.row == 0
    }

    pub fn is_at_left_edge(self) -> bool {
        self.column == 0
    }

    pub fn adjacent(self, dir: RelativeDir) -> Option<Self> {
        use RelativeDir::*;
        let Self { row, column, .. } = self;

        Some(match dir {
            TopLeft => Self::new(row.checked_sub(1)?, column.checked_sub(1)?),
            Left => Self::new(row, column.checked_sub(1)?),
            Right => Self::new(row, column + 1),
            TopRight => Self::new(row.checked_sub(1)?, column + 1),
            Top => Self::new(row.checked_sub(1)?, column),
            BottomLeft => Self::new(row + 1, column.checked_sub(1)?),
            Bottom => Self::new(row + 1, column),
            BottomRight => Self::new(row + 1, column + 1),
        })
    }

    // TODO: view ASM on x64 and WASM32; should be a noop.
    pub fn as_usize(self) -> (usize, usize) {
        (
            usize::try_from(self.row).unwrap(),
            usize::try_from(self.column).unwrap(),
        )
    }
}

impl<T: WorldObject> From<(u32, u32)> for WorldPosition<T> {
    fn from((row, column): (u32, u32)) -> Self {
        Self::new(row, column)
    }
}
