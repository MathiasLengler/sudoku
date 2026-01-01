use crate::{
    base::SudokuBase,
    cell::Candidates,
    error::{Error, Result},
    position::Coordinate,
    unsafe_utils::{get_unchecked, get_unchecked_mut},
};
use anyhow::anyhow;
use std::fmt::{Debug, Display};
use std::hash::Hash;

pub type CandidatesGroup<Base> = Group<Base, Candidates<Base>>;

/// Wrapper around `Base::Group<T>`, e.g. `[T; Base::SIDE_LENGTH]`.
///
/// Provides efficient indexing using `Coordinate<Base>` and better conversion errors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Group<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug> {
    group: Base::Group<T>,
}

impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default> Default for Group<Base, T> {
    fn default() -> Self {
        Self {
            group: Base::group_default(),
        }
    }
}

impl<Base: SudokuBase> Display for CandidatesGroup<Base> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (coordinate, candidates) in self.iter_enumerate() {
            writeln!(
                f,
                "{coordinate}:{:0width$b}",
                candidates.integral(),
                width = usize::from(Base::MAX_VALUE)
            )?;
        }

        Ok(())
    }
}

impl<Base: SudokuBase> Display for Group<Base, u8> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use itertools::Itertools;
        write!(f, "{}", self.iter().join(","))
    }
}

impl<Base: SudokuBase> Display for Group<Base, u32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use itertools::Itertools;
        write!(f, "{}", self.iter().join(","))
    }
}

impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default + Ord + Hash>
    Group<Base, T>
{
    pub fn new(group: Base::Group<T>) -> Self {
        Self { group }
    }

    // pub fn from_trusted_iter(iter: impl TrustedGroupSizeIter<Base, Item = T>) -> Self {
    //     Self {
    //         // TODO: optimize based on the safety contract of `TrustedGroupSizeIter`.
    //         group: iter.collect::<Vec<_>>().try_into().unwrap(),
    //     }
    // }

    pub fn from_iter_checked(iter: impl IntoIterator<Item = T>) -> Self {
        fn inner<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default + Ord + Hash>(
            iter: impl Iterator<Item = T>,
        ) -> Group<Base, T> {
            // TODO: use Base::group_uninit
            let iter = iter.into_iter();
            let mut this = Group::default();
            itertools::Itertools::zip_eq(this.iter_mut(), iter).for_each(
                |(group_item, iter_item)| {
                    *group_item = iter_item;
                },
            );
            this
        }

        inner(iter.into_iter())
    }

    pub fn get(&self, coordinate: Coordinate<Base>) -> T {
        // Safety:
        // - Coordinate::<Base>::get_usize: `coordinate < Base::SIDE_LENGTH`
        // - Base::Group<T>: array length equals `Base::SIDE_LENGTH`
        // Therefore the index remains in-bounds.
        *unsafe { get_unchecked(self.group.as_ref(), coordinate.get_usize()) }
    }

    pub fn get_mut(&mut self, coordinate: Coordinate<Base>) -> &mut T {
        // Safety:
        // - Coordinate::<Base>::get_usize: `coordinate < Base::SIDE_LENGTH`
        // - Base::Group<T>: array length equals `Base::SIDE_LENGTH`
        // Therefore the index remains in-bounds.
        unsafe { get_unchecked_mut(self.group.as_mut(), coordinate.get_usize()) }
    }

    pub fn as_slice(&self) -> &[T] {
        self.group.as_ref()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.group.as_mut()
    }

    pub fn map<F, U>(self, f: F) -> Group<Base, U>
    where
        F: FnMut(T) -> U,
        U: Send + Sync + Copy + Clone + Debug + Default + Ord + Hash,
    {
        Group {
            group: Base::group_map(self.group, f),
        }
    }

    #[must_use]
    pub fn reverse(mut self) -> Self {
        self.as_mut_slice().reverse();
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.group.as_ref().iter().copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.group.as_mut().iter_mut()
    }

    pub fn iter_enumerate(&self) -> impl Iterator<Item = (Coordinate<Base>, T)> + '_ {
        //TODO: evaluate zip_eq
        Coordinate::all().zip(self.iter())
    }

    pub fn iter_mut_enumerate(&mut self) -> impl Iterator<Item = (Coordinate<Base>, &mut T)> {
        //TODO: evaluate zip_eq
        Coordinate::all().zip(self.iter_mut())
    }

    pub fn into_iter_enumerate(self) -> impl Iterator<Item = (Coordinate<Base>, T)> {
        Coordinate::all().zip(self)
    }

    pub fn iter_index_mask(&self, index_mask: Candidates<Base>) -> impl Iterator<Item = T> + '_ {
        self.iter_enumerate()
            .filter(move |(coordinate, _t)| index_mask.has(*coordinate))
            .map(|(_coordinate, t)| t)
    }

    pub fn iter_mut_index_mask(
        &mut self,
        index_mask: Candidates<Base>,
    ) -> impl Iterator<Item = &mut T> {
        self.iter_mut_enumerate()
            .filter(move |(coordinate, _t)| index_mask.has(*coordinate))
            .map(|(_coordinate, t)| t)
    }
}

impl<Base: SudokuBase> CandidatesGroup<Base> {
    // TODO: optimize
    #[must_use]
    pub fn transpose(&self) -> CandidatesGroup<Base> {
        let mut transposed = Self::default();

        self.iter_enumerate().for_each(|(coordinate, candidates)| {
            candidates.iter().for_each(|candidate| {
                transposed.get_mut(candidate.into()).insert(coordinate);
            });
        });

        transposed
    }
}

impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default + Ord + Hash> IntoIterator
    for Group<Base, T>
{
    type Item = T;
    type IntoIter = <Base::Group<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.group.into_iter()
    }
}

impl<Base: SudokuBase, T: Send + Sync + Copy + Clone + Debug + Default + Ord + Hash> TryFrom<Vec<T>>
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

#[cfg(test)]
mod tests {
    use crate::{base::consts::*, cell::Value};

    use super::*;

    #[test]
    fn test_display_candidates_group() {
        let candidates_group: CandidatesGroup<Base3> = vec![
            vec![1, 2, 4],
            vec![2, 3, 4],
            vec![1, 3],
            vec![1, 4],
            vec![2, 5, 6, 9],
            vec![6, 3, 7, 9],
            vec![4, 6, 7, 8],
            vec![2, 3, 4, 5],
            vec![1, 2, 4, 6, 9],
        ]
        .into_iter()
        .map(Candidates::<Base3>::try_from)
        .collect::<Result<Vec<_>>>()
        .unwrap()
        .try_into()
        .unwrap();

        assert_eq!(
            format!("{candidates_group}"),
            "0:000001011
1:000001110
2:000000101
3:000001001
4:100110010
5:101100100
6:011101000
7:000011110
8:100101011
"
        );
    }

    #[test]
    fn test_display_u8_group() {
        let u8_group: Group<Base3, u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8].try_into().unwrap();

        assert_eq!(format!("{u8_group}"), "0,1,2,3,4,5,6,7,8");
    }

    #[test]
    fn test_from_iter_checked() {
        let group_data = vec![1, 2, 3, 4];
        let group = Group::<Base2, u8>::from_iter_checked(group_data.clone());
        itertools::assert_equal(group, group_data);
    }

    #[should_panic(expected = "itertools: .zip_eq() reached end of one iterator before the other")]
    #[test]
    fn test_from_iter_checked_panic() {
        Group::<Base2, u8>::from_iter_checked(vec![]);
    }

    #[test]
    fn test_get() {
        let group_data = vec![1, 2, 3, 4];
        let group: Group<Base2, u8> = group_data.clone().try_into().unwrap();

        assert_eq!(
            Coordinate::all()
                .map(|coordinate| group.get(coordinate))
                .collect::<Vec<_>>(),
            group_data
        );
    }

    #[test]
    fn test_get_mut() {
        let group_data = vec![1, 2, 3, 4];
        let mut group: Group<Base2, u8> = group_data.clone().try_into().unwrap();
        *group.get_mut(Coordinate::default()) = 5;
        itertools::assert_equal(group.iter(), vec![5, 2, 3, 4]);
        *group.get_mut(Coordinate::max()) = 6;
        itertools::assert_equal(group.iter(), vec![5, 2, 3, 6]);
    }

    #[test]
    fn test_map() {
        let group: CandidatesGroup<Base2> =
            Group::from_iter_checked(Candidates::iter_all_lexicographical().take(4));

        let group_counts = group.map(|candidates| candidates.count());
        itertools::assert_equal(group_counts.iter(), vec![0, 1, 1, 2]);
    }

    #[test]
    fn test_transpose() {
        let mut candidates_group = CandidatesGroup::<Base3>::default();
        *candidates_group.get_mut(Coordinate::default()) = Candidates::all();

        let transposed = candidates_group.transpose();

        transposed
            .iter()
            .for_each(|candidates| assert_eq!(candidates.to_single(), Some(Value::default())));

        assert_eq!(candidates_group, transposed.transpose());
    }
}
