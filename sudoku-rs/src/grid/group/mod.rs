use crate::{
    base::SudokuBase,
    cell::{Candidates, Value},
    error::{Error, Result},
    position::Coordinate,
    unsafe_utils::{get_unchecked, get_unchecked_mut},
};
use anyhow::anyhow;
use std::fmt::Debug;

pub(crate) type CandidatesGroup<Base> = Group<Base, Candidates<Base>>;

/// Wrapper around `Base::Group<T>`, e.g. `[T; Base::SIDE_LENGTH]`.
///
/// Provides efficient indexing using `Coordinate<Base>` and better conversion errors.
#[derive(Debug, Clone, Default)]
pub(crate) struct Group<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default> {
    group: Base::Group<T>,
}

impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default> Group<Base, T> {
    pub(crate) fn get(&self, coordinate: Coordinate<Base>) -> T {
        // Safety:
        // - Coordinate::<Base>::get_usize: `coordinate < Base::SIDE_LENGTH`
        // - Base::Group<T>: array length equals `Base::SIDE_LENGTH`
        // Therefore the index remains in-bounds.
        *unsafe { get_unchecked(self.group.as_ref(), coordinate.get_usize()) }
    }

    pub(crate) fn get_mut(&mut self, coordinate: Coordinate<Base>) -> &mut T {
        // Safety:
        // - Coordinate::<Base>::get_usize: `coordinate < Base::SIDE_LENGTH`
        // - Base::Group<T>: array length equals `Base::SIDE_LENGTH`
        // Therefore the index remains in-bounds.
        unsafe { get_unchecked_mut(self.group.as_mut(), coordinate.get_usize()) }
    }

    pub(crate) fn map<F, U>(self, f: F) -> Group<Base, U>
    where
        F: FnMut(T) -> U,
        U: Send + Sync + Copy + Clone + Debug + Default,
    {
        Group {
            group: self
                .group
                .into_iter()
                .map(f)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.group.as_ref().iter().copied()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.group.as_mut().iter_mut()
    }

    pub(crate) fn iter_enumerate(&self) -> impl Iterator<Item = (Coordinate<Base>, T)> + '_ {
        //TODO: evaluate zip_eq
        Coordinate::all().zip(self.iter())
    }

    pub(crate) fn iter_mut_enumerate(
        &mut self,
    ) -> impl Iterator<Item = (Coordinate<Base>, &mut T)> {
        //TODO: evaluate zip_eq
        Coordinate::all().zip(self.iter_mut())
    }

    pub(crate) fn iter_filter_mask(&self, mask: Candidates<Base>) -> impl Iterator<Item = T> + '_ {
        self.iter_enumerate()
            .filter(move |(coordinate, _t)| mask.has(Value::from(*coordinate)))
            .map(|(_coordinate, t)| t)
    }

    pub(crate) fn iter_mut_filter_mask(
        &mut self,
        mask: Candidates<Base>,
    ) -> impl Iterator<Item = &mut T> {
        self.iter_mut_enumerate()
            .filter(move |(coordinate, _t)| mask.has(Value::from(*coordinate)))
            .map(|(_coordinate, t)| t)
    }
}

impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default> IntoIterator
    for Group<Base, T>
{
    type Item = T;
    type IntoIter = <Base::Group<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.group.into_iter()
    }
}

impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default> TryFrom<Vec<T>>
    for Group<Base, T>
{
    type Error = Error;

    fn try_from(vec: Vec<T>) -> Result<Self> {
        Ok(Self {
            group: Base::Group::try_from(vec).map_err(|rejected_vec| {
                anyhow!(
                    "Invalid group length, expected {}, instead got {}",
                    Base::SIDE_LENGTH,
                    rejected_vec.len()
                )
            })?,
        })
    }
}
