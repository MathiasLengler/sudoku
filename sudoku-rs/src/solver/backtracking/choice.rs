use std::fmt::{self, Display};

use rand::{SeedableRng, thread_rng};
use rand::seq::SliceRandom;
use rand_xoshiro::Xoshiro256StarStar;

use crate::base::SudokuBase;
use crate::cell::Value;
use crate::solver::backtracking::CandidatesVisitOrder;

#[derive(Debug)]
pub(super) enum CandidatesProcessor {
    Asc,
    Desc,
    Random(Xoshiro256StarStar),
}

impl From<CandidatesVisitOrder> for CandidatesProcessor {
    fn from(candidates_visit_order: CandidatesVisitOrder) -> Self {
        match candidates_visit_order {
            CandidatesVisitOrder::Asc => CandidatesProcessor::Asc,
            CandidatesVisitOrder::Desc => CandidatesProcessor::Desc,
            CandidatesVisitOrder::Random => {
                CandidatesProcessor::Random(Xoshiro256StarStar::from_rng(thread_rng()).unwrap())
            }
            CandidatesVisitOrder::RandomSeed(seed) => {
                CandidatesProcessor::Random(Xoshiro256StarStar::seed_from_u64(seed))
            }
        }
    }
}

impl CandidatesProcessor {
    fn process<Base: SudokuBase>(&mut self, candidates: &mut [Value<Base>]) {
        match self {
            // Values are popped from the end of the candidates vec
            CandidatesProcessor::Asc => candidates.reverse(),
            CandidatesProcessor::Desc => {}
            CandidatesProcessor::Random(rng) => candidates.shuffle(rng),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Choice<Base: SudokuBase> {
    candidates: Vec<Value<Base>>,
}

impl<Base: SudokuBase> Choice<Base> {
    pub(super) fn new(
        mut candidates: Vec<Value<Base>>,
        candidates_processor: &mut CandidatesProcessor,
    ) -> Choice<Base> {
        candidates_processor.process(&mut candidates);
        Self { candidates }
    }

    pub(super) fn set_next(&mut self) {
        let prev_selection = self.candidates.pop();

        debug_assert!(prev_selection.is_some());
    }

    pub(super) fn is_exhausted(&self) -> bool {
        self.candidates.is_empty()
    }

    pub(super) fn selection(&self) -> Option<Value<Base>> {
        self.candidates.last().copied()
    }
}

impl<Base: SudokuBase> Display for Choice<Base> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", self.candidates)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::base::consts::*;

    use super::*;

    #[test]
    fn test_choice() {
        let mut choice = Choice::<Base2>::new(
            vec![1, 2, 4]
                .into_iter()
                .map(|v| v.try_into().unwrap())
                .collect(),
            &mut CandidatesProcessor::Asc,
        );

        assert_eq!(choice.selection(), Some(1.try_into().unwrap()));
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), Some(2.try_into().unwrap()));
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), Some(4.try_into().unwrap()));
        assert_eq!(choice.is_exhausted(), false);

        choice.set_next();
        assert_eq!(choice.selection(), None);
        assert_eq!(choice.is_exhausted(), true);
    }
}
