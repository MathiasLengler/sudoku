use std::{
    fmt::{self, Display, Formatter},
    num::NonZeroUsize,
};

use crate::error::Result;
use anyhow::{ensure, Context};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;
pub(in crate::world) use validated::ValidatedGridIndex;

mod validated {
    use super::*;

    /// A grid index that has been validated to be within bounds for some `WorldDim`.
    ///
    /// In contrast to `SudokuBase`-bounded types, this can't be relied on.
    /// It only works for indexing into a cells `Array2` created with the same `WorldDim`, but this is not enfored by the type system.
    ///
    /// The intended usage is inside `CellWorld`, to differentiate between use-provided (untrusted) and internally computed indexes.
    /// Since we don't provide a way to mutate the size of the world, this works in practice.
    #[derive(
        Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
    )]
    pub(in crate::world) struct ValidatedGridIndex(GridIndex);

    impl ValidatedGridIndex {
        pub(in crate::world) fn new(index: GridIndex, grid_dim: WorldDim) -> Result<Self> {
            ensure!(
                index.contained_in(grid_dim),
                "{index:?} out of bounds for {grid_dim:?}"
            );

            Ok(Self::new_unchecked(index))
        }

        pub(super) fn new_opt(index: GridIndex, grid_dim: WorldDim) -> Option<Self> {
            index
                .contained_in(grid_dim)
                .then(|| Self::new_unchecked(index))
        }

        pub(super) fn new_unchecked(index: GridIndex) -> Self {
            Self(index)
        }

        pub(in crate::world) fn get(self) -> GridIndex {
            self.0
        }

        pub(in crate::world) fn adjacent(
            self,
            dir: RelativeGridDir,
            grid_dim: WorldDim,
        ) -> Option<Self> {
            let GridIndex { row, column } = self.0;

            let adjacent = match dir {
                RelativeGridDir::TopLeft => GridIndex {
                    row: row.checked_sub(1)?,
                    column: column.checked_sub(1)?,
                },
                RelativeGridDir::Left => GridIndex {
                    row,
                    column: column.checked_sub(1)?,
                },
                RelativeGridDir::Right => GridIndex {
                    row,
                    column: column + 1,
                },
                RelativeGridDir::TopRight => GridIndex {
                    row: row.checked_sub(1)?,
                    column: column + 1,
                },
                RelativeGridDir::Top => GridIndex {
                    row: row.checked_sub(1)?,
                    column,
                },
                RelativeGridDir::BottomLeft => GridIndex {
                    row: row + 1,
                    column: column.checked_sub(1)?,
                },

                RelativeGridDir::Bottom => GridIndex {
                    row: row + 1,
                    column,
                },
                RelativeGridDir::BottomRight => GridIndex {
                    row: row + 1,
                    column: column + 1,
                },
            };

            Self::new_opt(adjacent, grid_dim)
        }
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GridIndex {
    pub row: usize,
    pub column: usize,
}

impl Display for GridIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { row, column } = self;
        write!(f, "({row}, {column})")
    }
}

impl GridIndex {
    pub fn contained_in(self, grid_dim: WorldDim) -> bool {
        grid_dim.contains(self)
    }

    pub(in crate::world) fn validate(self, grid_dim: WorldDim) -> Result<ValidatedGridIndex> {
        ValidatedGridIndex::new(self, grid_dim)
    }

    pub fn is_at_top_edge(self) -> bool {
        self.row == 0
    }

    pub fn is_at_left_edge(self) -> bool {
        self.column == 0
    }
}

/// Dimensions of a `CellWorld`.
/// Can represent either cells or grids.
#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldDim {
    pub row_count: NonZeroUsize,
    pub column_count: NonZeroUsize,
}

impl Display for WorldDim {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            row_count,
            column_count,
        } = self;
        write!(f, "({row_count}, {column_count})")
    }
}

impl WorldDim {
    pub fn new(row_count: usize, column_count: usize) -> Result<Self> {
        Ok(Self {
            row_count: NonZeroUsize::new(row_count).context("row_count must be non-zero")?,
            column_count: NonZeroUsize::new(column_count)
                .context("column_count must be non-zero")?,
        })
    }

    pub fn contains(self, index: GridIndex) -> bool {
        let WorldDim {
            row_count,
            column_count,
        } = self;
        let GridIndex { row, column } = index;

        (0..row_count.get()).contains(&row) && (0..column_count.get()).contains(&column)
    }

    pub fn all_indexes_count(self) -> usize {
        self.row_count.get() * self.column_count.get()
    }

    pub fn all_indexes(self) -> impl Iterator<Item = GridIndex> {
        (0..self.row_count.get()).flat_map(move |row| {
            (0..self.column_count.get()).map(move |column| GridIndex { row, column })
        })
    }

    pub(in crate::world) fn all_validated_indexes(
        self,
    ) -> impl Iterator<Item = ValidatedGridIndex> {
        self.all_indexes().map(ValidatedGridIndex::new_unchecked)
    }

    pub fn grid_count(self) -> usize {
        let WorldDim {
            row_count,
            column_count,
        } = self;

        row_count.get() * column_count.get()
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelativeGridDir {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl RelativeGridDir {
    pub fn all() -> impl Iterator<Item = Self> {
        use RelativeGridDir::*;

        [
            TopLeft,
            Top,
            TopRight,
            Left,
            Right,
            BottomLeft,
            Bottom,
            BottomRight,
        ]
        .into_iter()
    }
}
