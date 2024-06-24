use crate::{
    base::SudokuBase,
    cell::Candidates,
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

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.group.as_mut().iter_mut()
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
