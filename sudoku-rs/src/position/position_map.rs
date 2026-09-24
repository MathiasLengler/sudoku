use std::collections::btree_map::Iter;
use std::collections::{BTreeMap, btree_map};
use std::fmt::{Display, Formatter};
use std::iter::Map;

use crate::error::{Error, Result};
use crate::position::Position;
use crate::{base::SudokuBase, position::Positioned};

pub trait Merge: Sized + Copy {
    fn merge(self, other: Self) -> Result<Self>;
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PositionMap<Base: SudokuBase, T: Merge> {
    map: BTreeMap<Position<Base>, T>,
}

impl<Base: SudokuBase, T: Merge + Display> Display for PositionMap<Base, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use itertools::Itertools;

        write!(
            f,
            "{}",
            self.map
                .iter()
                .map(|(pos, action)| format!("{pos}: {action}"))
                .join(", ")
        )
    }
}

impl<Base: SudokuBase, T: Merge> Default for PositionMap<Base, T> {
    fn default() -> Self {
        Self::new()
    }
}

type PositionMapIntoIter<Base, T> =
    Map<btree_map::IntoIter<Position<Base>, T>, fn((Position<Base>, T)) -> Positioned<Base, T>>;

impl<Base: SudokuBase, T: Merge> IntoIterator for PositionMap<Base, T> {
    type Item = Positioned<Base, T>;
    type IntoIter = PositionMapIntoIter<Base, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter().map(Positioned::from)
    }
}

type PositionMapIter<'a, Base, T> =
    Map<Iter<'a, Position<Base>, T>, fn((&Position<Base>, &'a T)) -> Positioned<Base, &'a T>>;

impl<'a, Base: SudokuBase, T: Merge> IntoIterator for &'a PositionMap<Base, T> {
    type Item = Positioned<Base, &'a T>;
    type IntoIter = PositionMapIter<'a, Base, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Base: SudokuBase, T: Merge> PositionMap<Base, T> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::default(),
        }
    }

    pub fn with_single(pos: Position<Base>, value: T) -> Self {
        let mut this: Self = Self::new();
        this.map.insert(pos, value);
        this
    }

    pub fn try_from_iter(
        iter: impl IntoIterator<Item: TryInto<Positioned<Base, T>, Error: Into<Error>>>,
    ) -> Result<Self> {
        let mut this = Self::new();

        for into_positioned in iter {
            let Positioned { pos, value } = into_positioned.try_into().map_err(Into::into)?;
            this.insert(pos, value)?;
        }

        Ok(this)
    }

    pub fn merge(&mut self, other: Self) -> Result<()> {
        for Positioned { pos, value } in other {
            self.insert(pos, value)?;
        }
        Ok(())
    }

    pub fn iter(&self) -> PositionMapIter<'_, Base, T> {
        self.map
            .iter()
            .map(|(&pos, value)| Positioned { pos, value })
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn insert(&mut self, pos: Position<Base>, value: T) -> Result<()> {
        if let Some(existing_value) = self.map.get_mut(&pos) {
            *existing_value = (*existing_value).merge(value)?;
        } else {
            self.map.insert(pos, value);
        }

        Ok(())
    }
}
