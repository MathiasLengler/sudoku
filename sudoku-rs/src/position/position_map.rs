use std::collections::btree_map::Iter;
use std::collections::{BTreeMap, btree_map};
use std::fmt::{Display, Formatter};
use std::iter::Map;

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::position::Position;

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

impl<Base: SudokuBase, T: Merge> IntoIterator for PositionMap<Base, T> {
    type Item = (Position<Base>, T);
    type IntoIter = btree_map::IntoIter<Position<Base>, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

type PositionMapIter<'a, Base, T> =
    Map<Iter<'a, Position<Base>, T>, fn((&Position<Base>, &'a T)) -> (Position<Base>, &'a T)>;

impl<'a, Base: SudokuBase, T: Merge> IntoIterator for &'a PositionMap<Base, T> {
    type Item = (Position<Base>, &'a T);
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

    pub fn try_from_iter<I, IntoPos, IntoT>(iter: I) -> Result<Self>
    where
        I: IntoIterator<Item = (IntoPos, IntoT)>,
        IntoPos: TryInto<Position<Base>>,
        IntoT: TryInto<T>,
        Error: From<IntoPos::Error>,
        Error: From<IntoT::Error>,
    {
        let mut this = Self::new();

        for (pos, value) in iter {
            this.insert(pos.try_into()?, value.try_into()?)?;
        }

        Ok(this)
    }

    pub fn merge(&mut self, other: Self) -> Result<()> {
        for (pos, value) in other {
            self.insert(pos, value)?;
        }
        Ok(())
    }

    pub fn iter(&self) -> PositionMapIter<'_, Base, T> {
        self.map.iter().map(|(pos, value)| (*pos, value))
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
