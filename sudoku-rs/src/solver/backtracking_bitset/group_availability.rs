use crate::base::SudokuBase;
use crate::cell::{Candidates, Value};
use crate::position::{Coordinate, Position};
use crate::unsafe_utils::{get_unchecked, get_unchecked_mut};

/// A compact data structure representing group information of a sudoku grid.
///
/// Use-case: For each group index, what are the available candidates?
#[derive(Debug, Clone, Default)]
pub(crate) struct GroupAvailability<Base: SudokuBase> {
    rows: Base::CandidatesGroup,
    columns: Base::CandidatesGroup,
    blocks: Base::CandidatesGroup,
}

impl<Base: SudokuBase> GroupAvailability<Base> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn all() -> Self {
        let mut this = Self::new();

        this.iter_mut()
            .for_each(|candidates| *candidates = Candidates::all());

        this
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Candidates<Base>> {
        self.rows
            .as_mut()
            .iter_mut()
            .chain(self.columns.as_mut().iter_mut())
            .chain(self.blocks.as_mut().iter_mut())
    }

    pub(crate) fn reserve(&mut self, index: GroupAvailabilityIndex<Base>, candidate: Value<Base>) {
        // Clear candidate availability
        self.mutate(index, |candidates| {
            candidates.set(candidate, false);
        });
    }
    pub(crate) fn restore(&mut self, index: GroupAvailabilityIndex<Base>, candidate: Value<Base>) {
        // Restore candidate availability
        self.mutate(index, |candidates| {
            candidates.set(candidate, true);
        });
    }

    fn get(
        &self,
        index: GroupAvailabilityIndex<Base>,
    ) -> (Candidates<Base>, Candidates<Base>, Candidates<Base>) {
        let (row, column, block) = index.into_usize_tuple();

        // Safety: relies on invariants:
        // - Coordinate::<Base>::get: `coordinate < Base::SIDE_LENGTH`
        // - Base::CandidatesCells: array length equals `Base::SIDE_LENGTH`
        // Therefore the indexes remain in-bounds.
        let (row_candidates, column_candidates, block_candidates) = unsafe {
            (
                get_unchecked(self.rows.as_ref(), row),
                get_unchecked(self.columns.as_ref(), column),
                get_unchecked(self.blocks.as_ref(), block),
            )
        };
        (*row_candidates, *column_candidates, *block_candidates)
    }

    fn get_mut(
        &mut self,
        index: GroupAvailabilityIndex<Base>,
    ) -> (
        &mut Candidates<Base>,
        &mut Candidates<Base>,
        &mut Candidates<Base>,
    ) {
        let (row, column, block) = index.into_usize_tuple();

        // Safety: relies on invariants:
        // - Coordinate::<Base>::get: `coordinate < Base::SIDE_LENGTH`
        // - Base::CandidatesCells: array length equals `Base::SIDE_LENGTH`
        // Therefore the indexes remain in-bounds.
        let (row_candidates, column_candidates, block_candidates) = unsafe {
            (
                get_unchecked_mut(self.rows.as_mut(), row),
                get_unchecked_mut(self.columns.as_mut(), column),
                get_unchecked_mut(self.blocks.as_mut(), block),
            )
        };
        (row_candidates, column_candidates, block_candidates)
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
    pub(crate) fn intersection(&self, index: GroupAvailabilityIndex<Base>) -> Candidates<Base> {
        let (row_candidates, column_candidates, block_candidates) = self.get(index);

        row_candidates
            .intersection(column_candidates)
            .intersection(block_candidates)
    }
}

/// An index into `GroupAvailability`.
/// Logically it is just a `Position`, but contains a pre-computed `Coordinate` for each group.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub(crate) struct GroupAvailabilityIndex<Base: SudokuBase> {
    row: Coordinate<Base>,
    column: Coordinate<Base>,
    block: Coordinate<Base>,
}

impl<Base: SudokuBase> GroupAvailabilityIndex<Base> {
    fn into_usize_tuple(self) -> (usize, usize, usize) {
        let GroupAvailabilityIndex { row, column, block } = self;
        let row = usize::from(row.get());
        let column = usize::from(column.get());
        let block = usize::from(block.get());
        (row, column, block)
    }
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
