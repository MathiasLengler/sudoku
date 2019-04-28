use std::fmt::{self, Debug, Display, Formatter};
use std::num::NonZeroUsize;

use fixedbitset::FixedBitSet;

// TODO: candidates
// TODO: generic cell value type
pub trait SudokuCell: Default + Clone + Display + Debug + Ord + Eq + Send {
    fn new<I: IntoIterator<Item=usize>>(value: usize, max: usize, candidates: I) -> Self;
    fn new_with_value(value: usize, max: usize) -> Self;

    fn value(&self) -> usize;
    fn set_value(&mut self, value: usize, max: usize) -> usize;
    fn has_value(&self) -> bool;

    fn set_candidates<I: IntoIterator<Item=usize>>(&mut self, candidates: I, max: usize);
    fn candidates(&self) -> Vec<usize>;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Default, Debug)]
pub struct OptionCell {
    value: Option<NonZeroUsize>,
    candidates: FixedBitSet,
}

impl SudokuCell for OptionCell {
    fn new<I: IntoIterator<Item=usize>>(value: usize, max: usize, candidates: I) -> Self {
        Self {
            value: Self::import_value(value, max),
            candidates: Self::import_candidates(candidates, max),
        }
    }

    fn new_with_value(value: usize, max: usize) -> Self {
        Self::new(value, max, vec![])
    }

    fn value(&self) -> usize {
        Self::export_value(&self.value)
    }

    fn set_value(&mut self, value: usize, max: usize) -> usize {
        use std::mem::replace;

        let new_value = Self::import_value(value, max);

        let old_value = replace(&mut self.value, new_value);

        Self::export_value(&old_value)
    }

    fn has_value(&self) -> bool {
        self.value.is_some()
    }

    fn set_candidates<I: IntoIterator<Item=usize>>(&mut self, candidates: I, max: usize) {
        self.candidates = Self::import_candidates(candidates, max)
    }

    fn candidates(&self) -> Vec<usize> {
        Self::export_candidates(&self.candidates)
    }
}

/// Conversion Helpers
impl OptionCell {
    fn import_value(value: usize, max: usize) -> Option<NonZeroUsize> {
        Self::assert_value(value, max);

        NonZeroUsize::new(value)
    }
    fn export_value(value: &Option<NonZeroUsize>) -> usize {
        match value {
            Some(value) => value.get(),
            None => 0,
        }
    }
    fn import_candidates<I: IntoIterator<Item=usize>>(candidates: I, max: usize) -> FixedBitSet {
        candidates.into_iter().map(|candidate| {
            Self::assert_candidate(candidate, max);

            candidate
        }).collect()
    }
    fn export_candidates(candidates: &FixedBitSet) -> Vec<usize> {
        candidates.ones().collect()
    }

    // TODO: Check value of cells
    // TODO: assert all entry points
    fn assert_value(value: usize, max: usize) {
        assert!(value <= max);
    }

    fn assert_candidate(candidate: usize, max: usize) {
        assert!(candidate != 0 && candidate <= max);
    }
}

impl Display for OptionCell {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(&match self.value {
            None => "_".to_string(),
            Some(value) => value.to_string(),
        })
    }
}
