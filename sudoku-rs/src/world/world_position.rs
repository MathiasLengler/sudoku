use std::{
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
    marker::PhantomData,
    num::NonZeroUsize,
};

use crate::error::Result;
use anyhow::{ensure, Context};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;
// TODO: use for cell indexing of CellWorld
#[allow(unused_imports)]
pub(in crate::world) use validated::ValidatedWorldCellPosition;
pub(in crate::world) use validated::ValidatedWorldGridPosition;
pub(in crate::world) use validated::ValidatedWorldPosition;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CellMarker;
impl WorldObject for CellMarker {}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GridMarker;
impl WorldObject for GridMarker {}

pub trait WorldObject
where
    Self: Ord + Hash + Clone + Copy + Debug + Default + Send + Sync + 'static,
{
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export), ts(concrete(T = CellMarker)))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct WorldPosition<T: WorldObject> {
    pub row: usize,
    pub column: usize,
    #[ts(skip)]
    object: PhantomData<T>,
}

pub type WorldCellPosition = WorldPosition<CellMarker>;
pub type WorldGridPosition = WorldPosition<GridMarker>;

impl<T: WorldObject> Display for WorldPosition<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { row, column, .. } = self;
        write!(f, "({row}, {column})")
    }
}

impl<T: WorldObject> WorldPosition<T> {
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            object: PhantomData,
        }
    }

    pub fn contained_in(self, grid_dim: WorldDim<T>) -> bool {
        grid_dim.contains(self)
    }

    pub(in crate::world) fn validate(
        self,
        grid_dim: WorldDim<T>,
    ) -> Result<ValidatedWorldPosition<T>> {
        ValidatedWorldPosition::new(self, grid_dim)
    }

    pub fn is_at_top_edge(self) -> bool {
        self.row == 0
    }

    pub fn is_at_left_edge(self) -> bool {
        self.column == 0
    }
}

mod validated {
    use super::*;

    /// A world position that has been validated to be within bounds for some `WorldDim`.
    ///
    /// In contrast to `SudokuBase`-bounded types, this can't be relied on by unsafe code.
    ///
    /// The position is validated against a provided `WorldDim` when created, but could in principle be used with any `WorldDim`.
    /// The intended usage is inside `CellWorld`,
    /// in order to differentiate between use-provided (untrusted) and internally computed positions.
    /// Since we don't provide a way to mutate the size of the world *and* the type does not escape a world instance because of its visibility,
    /// this hopefully works in practice.
    #[derive(
        Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
    )]
    pub(in crate::world) struct ValidatedWorldPosition<T: WorldObject>(WorldPosition<T>);

    #[allow(dead_code)]
    pub(in crate::world) type ValidatedWorldCellPosition = ValidatedWorldPosition<CellMarker>;
    pub(in crate::world) type ValidatedWorldGridPosition = ValidatedWorldPosition<GridMarker>;

    impl<T: WorldObject> ValidatedWorldPosition<T> {
        pub(in crate::world) fn new(
            index: WorldPosition<T>,
            grid_dim: WorldDim<T>,
        ) -> Result<Self> {
            ensure!(
                index.contained_in(grid_dim),
                "{index:?} out of bounds for {grid_dim:?}"
            );

            Ok(Self::new_unchecked(index))
        }

        pub(super) fn new_opt(index: WorldPosition<T>, grid_dim: WorldDim<T>) -> Option<Self> {
            index
                .contained_in(grid_dim)
                .then(|| Self::new_unchecked(index))
        }

        pub(super) fn new_unchecked(index: WorldPosition<T>) -> Self {
            Self(index)
        }

        pub(in crate::world) fn get(self) -> WorldPosition<T> {
            self.0
        }

        pub(in crate::world) fn adjacent(
            self,
            dir: RelativeDir,
            grid_dim: WorldDim<T>,
        ) -> Option<Self> {
            let WorldPosition { row, column, .. } = self.0;

            let adjacent: WorldPosition<T> = match dir {
                RelativeDir::TopLeft => {
                    WorldPosition::new(row.checked_sub(1)?, column.checked_sub(1)?)
                }
                RelativeDir::Left => WorldPosition::new(row, column.checked_sub(1)?),
                RelativeDir::Right => WorldPosition::new(row, column + 1),
                RelativeDir::TopRight => WorldPosition::new(row.checked_sub(1)?, column + 1),
                RelativeDir::Top => WorldPosition::new(row.checked_sub(1)?, column),
                RelativeDir::BottomLeft => WorldPosition::new(row + 1, column.checked_sub(1)?),
                RelativeDir::Bottom => WorldPosition::new(row + 1, column),
                RelativeDir::BottomRight => WorldPosition::new(row + 1, column + 1),
            };

            Self::new_opt(adjacent, grid_dim)
        }
    }
}

/// Dimensions of a `CellWorld`.
/// Can represent either cells or grids.
#[cfg_attr(feature = "wasm", derive(TS), ts(export), ts(concrete(T = CellMarker)))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldDim<T: WorldObject> {
    pub row_count: NonZeroUsize,
    pub column_count: NonZeroUsize,
    #[ts(skip)]
    object: PhantomData<T>,
}

impl<T: WorldObject> Display for WorldDim<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            row_count,
            column_count,
            ..
        } = self;
        write!(f, "({row_count}, {column_count})")
    }
}

pub type WorldCellDim = WorldDim<CellMarker>;
pub type WorldGridDim = WorldDim<GridMarker>;

impl<T: WorldObject> WorldDim<T> {
    pub fn new(row_count: usize, column_count: usize) -> Result<Self> {
        Ok(Self {
            row_count: NonZeroUsize::new(row_count).context("row_count must be non-zero")?,
            column_count: NonZeroUsize::new(column_count)
                .context("column_count must be non-zero")?,
            object: PhantomData,
        })
    }

    pub fn contains(self, position: WorldPosition<T>) -> bool {
        let WorldDim {
            row_count,
            column_count,
            ..
        } = self;
        let WorldPosition { row, column, .. } = position;

        (0..row_count.get()).contains(&row) && (0..column_count.get()).contains(&column)
    }

    pub fn all_indexes_count(self) -> usize {
        self.row_count.get() * self.column_count.get()
    }

    pub fn all_indexes(self) -> impl Iterator<Item = WorldPosition<T>> {
        (0..self.row_count.get()).flat_map(move |row| {
            (0..self.column_count.get()).map(move |column| WorldPosition::new(row, column))
        })
    }

    pub(in crate::world) fn all_validated_indexes(
        self,
    ) -> impl Iterator<Item = ValidatedWorldPosition<T>> {
        self.all_indexes()
            .map(ValidatedWorldPosition::new_unchecked)
    }

    pub fn object_count(self) -> usize {
        let WorldDim {
            row_count,
            column_count,
            ..
        } = self;

        row_count.get() * column_count.get()
    }
}

// TODO: conversions
impl WorldCellDim {
    pub fn cell_count(self) -> usize {
        self.object_count()
    }
}

impl WorldGridDim {
    pub fn grid_count(self) -> usize {
        self.object_count()
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelativeDir {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl RelativeDir {
    pub fn all() -> impl Iterator<Item = Self> {
        use RelativeDir::*;

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
