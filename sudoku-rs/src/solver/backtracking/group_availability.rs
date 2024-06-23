use crate::base::SudokuBase;
use crate::cell::{Candidates, Value};
use crate::position::{Coordinate, Position};
use crate::solver::backtracking::candidates_filter::CandidatesFilter;
use crate::unsafe_utils::{get_unchecked, get_unchecked_mut};

/// A thin indexing wrapper around a `[Candidates<Base>; Base::SIDE_LENGTH]`.
#[derive(Debug, Clone, Default)]
pub(crate) struct CandidatesGroup<Base: SudokuBase> {
    candidates_group: Base::CandidatesGroup,
}

impl<Base: SudokuBase> CandidatesGroup<Base> {
    pub(crate) fn get(&self, coordinate: Coordinate<Base>) -> Candidates<Base> {
        // Safety:
        // - Coordinate::<Base>::get_usize: `coordinate < Base::SIDE_LENGTH`
        // - Base::CandidatesCells: array length equals `Base::SIDE_LENGTH`
        // Therefore the index remains in-bounds.
        *unsafe { get_unchecked(self.candidates_group.as_ref(), coordinate.get_usize()) }
    }

    pub(crate) fn get_mut(&mut self, coordinate: Coordinate<Base>) -> &mut Candidates<Base> {
        // Safety:
        // - Coordinate::<Base>::get_usize: `coordinate < Base::SIDE_LENGTH`
        // - Base::CandidatesCells: array length equals `Base::SIDE_LENGTH`
        // Therefore the index remains in-bounds.
        unsafe { get_unchecked_mut(self.candidates_group.as_mut(), coordinate.get_usize()) }
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Candidates<Base>> {
        self.candidates_group.as_mut().iter_mut()
    }
}

impl<Base: SudokuBase> IntoIterator for CandidatesGroup<Base> {
    type Item = Candidates<Base>;
    type IntoIter = Base::CandidatesGroupIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.candidates_group.into_iter()
    }
}

/// A compact data structure representing group information of a sudoku grid.
///
/// Saves `Base::SIDE_LENGTH` bits of information for each group.
///
/// Use-cases:
/// - What are the available candidates at a position?
/// - Where in each group is a specific candidate set?
#[derive(Debug, Clone, Default)]
pub(crate) struct GroupAvailability<Base: SudokuBase, Filter: CandidatesFilter<Base>> {
    rows: CandidatesGroup<Base>,
    columns: CandidatesGroup<Base>,
    blocks: CandidatesGroup<Base>,
    pub(crate) filter: Filter,
}

impl<Base: SudokuBase> GroupAvailability<Base, ()> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn all() -> Self {
        let mut this = Self::new();
        this.iter_mut()
            .for_each(|candidates| *candidates = Candidates::all());
        this
    }

    pub(crate) fn with_filter<Filter: CandidatesFilter<Base>>(
        self,
        filter: Filter,
    ) -> GroupAvailability<Base, Filter> {
        let GroupAvailability {
            rows,
            columns,
            blocks,
            filter: (),
        } = self;

        GroupAvailability {
            rows,
            columns,
            blocks,
            filter,
        }
    }
}

impl<Base: SudokuBase, Filter: CandidatesFilter<Base>> GroupAvailability<Base, Filter> {
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Candidates<Base>> {
        self.rows
            .iter_mut()
            .chain(self.columns.iter_mut())
            .chain(self.blocks.iter_mut())
    }

    pub(crate) fn delete(&mut self, index: GroupAvailabilityIndex<Base>, candidate: Value<Base>) {
        self.mutate(index, |candidates| candidates.delete(candidate));
    }
    pub(crate) fn insert(&mut self, index: GroupAvailabilityIndex<Base>, candidate: Value<Base>) {
        self.mutate(index, |candidates| candidates.insert(candidate));
    }

    fn get(
        &self,
        index: GroupAvailabilityIndex<Base>,
    ) -> (Candidates<Base>, Candidates<Base>, Candidates<Base>) {
        (
            self.rows.get(index.row),
            self.columns.get(index.column),
            self.blocks.get(index.block),
        )
    }

    fn get_mut(
        &mut self,
        index: GroupAvailabilityIndex<Base>,
    ) -> (
        &mut Candidates<Base>,
        &mut Candidates<Base>,
        &mut Candidates<Base>,
    ) {
        (
            self.rows.get_mut(index.row),
            self.columns.get_mut(index.column),
            self.blocks.get_mut(index.block),
        )
    }

    fn mutate(
        &mut self,
        index: GroupAvailabilityIndex<Base>,
        mut f: impl FnMut(&mut Candidates<Base>),
    ) {
        let (row_candidates, column_candidates, block_candidates) = self.get_mut(index);

        f(row_candidates);
        f(column_candidates);
        f(block_candidates);
    }

    /// Calculate the available (direct) candidates for a given index.
    /// This is calculated by intersecting the availability across the three different groups.
    pub(crate) fn available_candidates_at(
        &self,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        let (row_candidates, column_candidates, block_candidates) = self.get(index);

        let available_candidates = row_candidates
            .intersection(column_candidates)
            .intersection(block_candidates);

        available_candidates.without(self.filter.denied_candidates(index.into()))
    }
}

/// An index into `GroupAvailability`.
/// Logically it is just a `Position`, but contains a pre-computed `Coordinate` for each group.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct GroupAvailabilityIndex<Base: SudokuBase> {
    row: Coordinate<Base>,
    column: Coordinate<Base>,
    block: Coordinate<Base>,
}

impl<Base: SudokuBase> From<Position<Base>> for GroupAvailabilityIndex<Base> {
    fn from(pos: Position<Base>) -> Self {
        let (row, column) = pos.to_row_and_column();

        GroupAvailabilityIndex {
            row,
            column,
            block: pos.to_block(),
        }
    }
}

impl<Base: SudokuBase> From<GroupAvailabilityIndex<Base>> for Position<Base> {
    fn from(index: GroupAvailabilityIndex<Base>) -> Self {
        (index.row, index.column).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    type Base = Base3;

    fn assert_all_eq_candidates_tuple(
        candidates_tuple: (Candidates<Base>, Candidates<Base>, Candidates<Base>),
        expected_candidates: Candidates<Base>,
    ) {
        assert_eq!(
            candidates_tuple,
            (
                expected_candidates,
                expected_candidates,
                expected_candidates
            )
        );
    }

    mod group_availability {
        use std::mem::size_of;

        use super::*;

        #[test]
        fn test_new() {
            let group_availability = GroupAvailability::<Base, ()>::new();

            let expected_candidates = Candidates::new();
            for pos in Position::<Base>::all() {
                let index = pos.into();
                let candidates_tuple = group_availability.get(index);
                assert_all_eq_candidates_tuple(candidates_tuple, expected_candidates);
            }
        }

        #[test]
        fn test_all() {
            let group_availability = GroupAvailability::<Base, ()>::all();

            let expected_candidates = Candidates::all();
            for pos in Position::<Base>::all() {
                let index = pos.into();
                let candidates_tuple = group_availability.get(index);
                assert_all_eq_candidates_tuple(candidates_tuple, expected_candidates);
            }
        }

        #[test]
        fn test_insert_delete_single() {
            let mut group_availability = GroupAvailability::<Base, ()>::new();

            let candidate = Value::<Base>::try_from(3).unwrap();

            let expected_empty_candidates = Candidates::new();
            let expected_single_candidates = Candidates::with_single(candidate);

            for pos in Position::<Base>::all() {
                let index = pos.into();

                let candidates_tuple = group_availability.get(index);
                assert_all_eq_candidates_tuple(candidates_tuple, expected_empty_candidates);

                group_availability.insert(index, candidate);

                let candidates_tuple = group_availability.get(index);
                assert_all_eq_candidates_tuple(candidates_tuple, expected_single_candidates);

                group_availability.delete(index, candidate);

                let candidates_tuple = group_availability.get(index);
                assert_all_eq_candidates_tuple(candidates_tuple, expected_empty_candidates);
            }
        }

        #[test]
        fn test_insert_delete_multiple() {
            type TestCase = (
                Position<Base>,
                Value<Base>,
                (Candidates<Base>, Candidates<Base>, Candidates<Base>),
            );

            let mut group_availability = GroupAvailability::<Base, ()>::new();

            let test_cases: Vec<TestCase> = vec![
                ((0, 0), 1, (vec![1, 2], vec![1, 3], vec![1, 2])),
                ((0, 1), 2, (vec![1, 2], vec![2], vec![1, 2])), // same row/block as pos (0, 0)
                ((3, 0), 3, (vec![3], vec![1, 3], vec![3])),    // same column as pos (0, 0)
                ((8, 8), 4, (vec![4], vec![4], vec![4])), // different row/column/block as rest
            ]
            .into_iter()
            .map(
                |(
                    (row, column),
                    candidate,
                    (row_candidates, column_candidates, block_candidates),
                )| {
                    (
                        (row, column).try_into().unwrap(),
                        candidate.try_into().unwrap(),
                        (
                            row_candidates.try_into().unwrap(),
                            column_candidates.try_into().unwrap(),
                            block_candidates.try_into().unwrap(),
                        ),
                    )
                },
            )
            .collect();

            for (pos, candidate, _) in test_cases.iter().copied() {
                group_availability.insert(pos.into(), candidate);
            }

            for (pos, _, expected_candidates_tuple) in test_cases {
                let candidates_tuple = group_availability.get(pos.into());
                assert_eq!(candidates_tuple, expected_candidates_tuple);
            }
        }

        #[test]
        fn test_size_of() {
            fn expected_size<Base: SudokuBase>() -> usize {
                3 * (usize::from(Base::SIDE_LENGTH) * size_of::<Candidates<Base>>())
            }

            let expected_sizes = vec![12, 54, 96, 300];

            assert_eq!(
                vec![
                    size_of::<GroupAvailability<Base2, ()>>(),
                    size_of::<GroupAvailability<Base3, ()>>(),
                    size_of::<GroupAvailability<Base4, ()>>(),
                    size_of::<GroupAvailability<Base5, ()>>()
                ],
                expected_sizes
            );

            assert_eq!(
                vec![
                    expected_size::<Base2>(),
                    expected_size::<Base3>(),
                    expected_size::<Base4>(),
                    expected_size::<Base5>(),
                ],
                expected_sizes
            );
        }
    }

    mod group_availability_index {
        use super::*;

        #[test]
        fn test_position_roundtrip() {
            for pos in Position::<Base>::all() {
                assert_eq!(pos, Position::from(GroupAvailabilityIndex::from(pos)));
            }
        }
    }
}
