use crate::{
    base::SudokuBase,
    cell::{Candidates, Value},
    grid::group::CandidatesGroup,
    position::Coordinate,
};

use group_cell_variable::GroupCellVariable;
use varisat::Lit;
mod group_cell_variable;

type Clause = Vec<Lit>;

// adapter for previous API
pub(super) fn reduce_candidates_group<Base: SudokuBase>(
    candidates_group: &[Candidates<Base>],
) -> Vec<Candidates<Base>> {
    let taken_candidates = candidates_group
        .iter()
        .fold(Candidates::new(), |acc, &candidates| acc.union(candidates));

    let missing_candidates = taken_candidates.invert();

    let mut candidates_group_vec = Vec::with_capacity(Base::SIDE_LENGTH.into());
    candidates_group_vec.extend(missing_candidates.into_iter().map(Candidates::with_single));
    candidates_group_vec.extend(candidates_group);

    let candidates_group = candidates_group_vec
        .try_into()
        .expect("Candidates group to be well formed");

    reduce_real_candidates_group::<Base>(candidates_group)
}

fn reduce_real_candidates_group<Base: SudokuBase>(
    candidates_group: CandidatesGroup<Base>,
) -> Vec<Candidates<Base>> {
    dbg!(candidates_group);

    todo!("implement using varisat SAT solver")
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base3;

    use super::*;

    #[ignore]
    #[test]
    fn test_name() {
        reduce_candidates_group::<Base3>(&[
            //
            Candidates::with_single(1.try_into().unwrap()),
            Candidates::with_single(3.try_into().unwrap()),
        ]);
    }
}
