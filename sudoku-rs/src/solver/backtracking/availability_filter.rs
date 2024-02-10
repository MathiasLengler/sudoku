use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::grid::Grid;
use crate::solver::backtracking::GroupAvailabilityIndex;

pub type DeniedCandidatesGrid<Base> = Grid<Base, Candidates<Base>>;

pub trait AvailabilityFilter<Base: SudokuBase> {
    /// For a given `available_candidates` at `index`, return `available_candidates` with 0 or more candidates removed.
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base>;
}

// No-op
impl<Base: SudokuBase> AvailabilityFilter<Base> for () {
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        _index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        available_candidates
    }
}

// A function can modify `available_candidates`
impl<
        Base: SudokuBase,
        F: Fn(Candidates<Base>, GroupAvailabilityIndex<Base>) -> Candidates<Base>,
    > AvailabilityFilter<Base> for F
{
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        self(available_candidates, index)
    }
}

impl<Base: SudokuBase> AvailabilityFilter<Base> for DeniedCandidatesGrid<Base> {
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        let denied_candidates = *self.get(index.into());

        available_candidates.without(denied_candidates)
    }
}

// A optional grid of candidates defines denied candidates for each position.
impl<Base: SudokuBase> AvailabilityFilter<Base> for Option<DeniedCandidatesGrid<Base>> {
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        if let Some(denylist) = self {
            denylist.filter(available_candidates, index)
        } else {
            available_candidates
        }
    }
}
