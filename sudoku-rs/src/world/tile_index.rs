// TODO: rename tile to grid
//  using grid and tile while referring to the same thing is confusing
//  we use grid everywhere, keep it consistent

use std::{
    fmt::{self, Display, Formatter},
    num::NonZeroUsize,
};

use crate::error::Result;
use anyhow::ensure;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;
pub(in crate::world) use validated::ValidatedTileIndex;

mod validated {
    use super::*;

    /// A tile index that has been validated to be within bounds for some `TileDim`.
    ///
    /// In contrast to `SudokuBase`-bounded types, this can't be relied on.
    /// It only works for indexing into a cells `Array2` created with the same `TileDim`, but this is not enfored by the type system.
    ///
    /// The intended usage is inside `CellWorld`, to differentiate between use-provided (untrusted) and internally computed indexes.
    /// Since we don't provide a way to mutate the size of the world, this works in practice.
    #[derive(
        Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
    )]
    pub(in crate::world) struct ValidatedTileIndex(TileIndex);

    impl ValidatedTileIndex {
        pub(in crate::world) fn new(index: TileIndex, tile_dim: TileDim) -> Result<Self> {
            ensure!(
                index.contained_in(tile_dim),
                "{index:?} out of bounds for {tile_dim:?}"
            );

            Ok(Self::new_unchecked(index))
        }

        pub(super) fn new_opt(index: TileIndex, tile_dim: TileDim) -> Option<Self> {
            index
                .contained_in(tile_dim)
                .then(|| Self::new_unchecked(index))
        }

        pub(super) fn new_unchecked(index: TileIndex) -> Self {
            Self(index)
        }

        pub(in crate::world) fn get(self) -> TileIndex {
            self.0
        }

        pub(in crate::world) fn adjacent(
            self,
            dir: RelativeTileDir,
            tile_dim: TileDim,
        ) -> Option<Self> {
            let TileIndex { row, column } = self.0;

            let adjacent = match dir {
                RelativeTileDir::TopLeft => TileIndex {
                    row: row.checked_sub(1)?,
                    column: column.checked_sub(1)?,
                },
                RelativeTileDir::Left => TileIndex {
                    row,
                    column: column.checked_sub(1)?,
                },
                RelativeTileDir::Right => TileIndex {
                    row,
                    column: column + 1,
                },
                RelativeTileDir::TopRight => TileIndex {
                    row: row.checked_sub(1)?,
                    column: column + 1,
                },
                RelativeTileDir::Top => TileIndex {
                    row: row.checked_sub(1)?,
                    column,
                },
                RelativeTileDir::BottomLeft => TileIndex {
                    row: row + 1,
                    column: column.checked_sub(1)?,
                },

                RelativeTileDir::Bottom => TileIndex {
                    row: row + 1,
                    column,
                },
                RelativeTileDir::BottomRight => TileIndex {
                    row: row + 1,
                    column: column + 1,
                },
            };

            Self::new_opt(adjacent, tile_dim)
        }
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TileIndex {
    pub row: usize,
    pub column: usize,
}

impl Display for TileIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { row, column } = self;
        write!(f, "({row}, {column})")
    }
}

impl TileIndex {
    pub fn contained_in(self, tile_dim: TileDim) -> bool {
        tile_dim.contains(self)
    }

    pub(in crate::world) fn validate(self, tile_dim: TileDim) -> Result<ValidatedTileIndex> {
        ValidatedTileIndex::new(self, tile_dim)
    }

    pub fn is_at_top_edge(self) -> bool {
        self.row == 0
    }

    pub fn is_at_left_edge(self) -> bool {
        self.column == 0
    }
}

/// How many tiles/sudoku grids are in the world
#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileDim {
    pub row_count: NonZeroUsize,
    pub column_count: NonZeroUsize,
}

impl Display for TileDim {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            row_count,
            column_count,
        } = self;
        write!(f, "({row_count}, {column_count})")
    }
}

impl TileDim {
    pub fn contains(self, index: TileIndex) -> bool {
        let TileDim {
            row_count,
            column_count,
        } = self;
        let TileIndex { row, column } = index;

        (0..row_count.get()).contains(&row) && (0..column_count.get()).contains(&column)
    }

    pub fn all_indexes(self) -> impl Iterator<Item = TileIndex> {
        (0..self.row_count.get()).flat_map(move |row| {
            (0..self.column_count.get()).map(move |column| TileIndex { row, column })
        })
    }

    pub(in crate::world) fn all_validated_indexes(
        self,
    ) -> impl Iterator<Item = ValidatedTileIndex> {
        self.all_indexes().map(ValidatedTileIndex::new_unchecked)
    }

    pub fn tile_count(self) -> usize {
        let TileDim {
            row_count,
            column_count,
        } = self;

        row_count.get() * column_count.get()
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelativeTileDir {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl RelativeTileDir {
    pub fn all() -> impl Iterator<Item = Self> {
        use RelativeTileDir::*;

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
