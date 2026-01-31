use std::{
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    num::NonZeroUsize,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    world::{ValidatedWorldPosition, WorldObject, WorldPosition},
};

/// Dimensions of a `CellWorld`.
/// Can represent either cells or grids.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export), ts(concrete(T = crate::world::CellMarker)))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldDim<T: WorldObject> {
    pub row_count: NonZeroUsize,
    pub column_count: NonZeroUsize,
    #[cfg_attr(feature = "wasm", ts(skip))]
    #[serde(skip)]
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

    pub fn all_positions_count(self) -> usize {
        self.row_count.get() * self.column_count.get()
    }

    pub fn all_positions(self) -> impl Iterator<Item = WorldPosition<T>> {
        (0..self.row_count.get()).flat_map(move |row| {
            (0..self.column_count.get()).map(move |column| WorldPosition::new(row, column))
        })
    }

    pub(in crate::world) fn all_validated_positions(
        self,
    ) -> impl Iterator<Item = ValidatedWorldPosition<T>> {
        self.all_positions()
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

    /// Iterate over all rows, yielding iterators of positions in each row.
    ///
    /// Returns an iterator where each item represents a row and yields positions
    /// in that row from left to right (column 0 to column_count - 1).
    pub fn all_rows(self) -> impl Iterator<Item = impl Iterator<Item = WorldPosition<T>>> {
        let column_count = self.column_count.get();
        (0..self.row_count.get()).map(move |row| {
            (0..column_count).map(move |column| WorldPosition::new(row, column))
        })
    }

    /// Iterate over all columns, yielding iterators of positions in each column.
    ///
    /// Returns an iterator where each item represents a column and yields positions
    /// in that column from top to bottom (row 0 to row_count - 1).
    pub fn all_columns(self) -> impl Iterator<Item = impl Iterator<Item = WorldPosition<T>>> {
        let row_count = self.row_count.get();
        (0..self.column_count.get()).map(move |column| {
            (0..row_count).map(move |row| WorldPosition::new(row, column))
        })
    }
}
