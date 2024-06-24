use crate::{
    base::SudokuBase,
    cell::Candidates,
    position::Coordinate,
    unsafe_utils::{get_unchecked, get_unchecked_mut},
};

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
    type IntoIter = <Base::CandidatesGroup as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.candidates_group.into_iter()
    }
}
