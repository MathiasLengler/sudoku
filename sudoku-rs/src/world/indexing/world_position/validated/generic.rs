use anyhow::ensure;

use std::fmt::Debug;

use crate::{
    error::Result,
    world::{RelativeDir, WorldDim, WorldObject, WorldPosition},
};

/// A world position that has been validated to be within bounds for some `WorldDim`.
///
/// In contrast to `SudokuBase`-bounded types, this can't be relied on by unsafe code.
///
/// The position is validated against a provided `WorldDim` when created, but could in principle be used with any `WorldDim`.
/// The intended usage is inside `CellWorld`,
/// in order to differentiate between use-provided (untrusted) and internally computed positions.
/// Since we don't provide a way to mutate the size of the world *and* the type does not escape a world instance because of its visibility,
/// this hopefully works in practice.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::world) struct ValidatedWorldPosition<T: WorldObject>(WorldPosition<T>);

impl<T: WorldObject> ValidatedWorldPosition<T> {
    pub(in crate::world) fn new(index: WorldPosition<T>, world_dim: WorldDim<T>) -> Result<Self> {
        ensure!(
            index.contained_in(world_dim),
            "{index:?} out of bounds for {world_dim:?}"
        );

        Ok(Self::new_unchecked(index))
    }

    pub(in crate::world::indexing) fn new_opt(
        index: WorldPosition<T>,
        world_dim: WorldDim<T>,
    ) -> Option<Self> {
        index
            .contained_in(world_dim)
            .then(|| Self::new_unchecked(index))
    }

    pub(in crate::world::indexing) fn new_unchecked(index: WorldPosition<T>) -> Self {
        Self(index)
    }

    pub(in crate::world) fn get(self) -> WorldPosition<T> {
        self.0
    }

    pub(in crate::world) fn adjacent(
        self,
        dir: RelativeDir,
        world_dim: WorldDim<T>,
    ) -> Option<Self> {
        let adjacent = self.get().adjacent(dir)?;

        Self::new_opt(adjacent, world_dim)
    }
}
