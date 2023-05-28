use std::cmp::Eq;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub(crate) enum CellState<Base: SudokuBase> {
    Value(Value<Base>),
    FixedValue(Value<Base>),
    Candidates(Candidates<Base>),
}

impl<Base: SudokuBase> Default for CellState<Base> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Base: SudokuBase> CellState<Base> {
    pub(super) fn new() -> Self {
        Self::with_candidates(Candidates::new())
    }

    pub(super) fn with_value(value: Value<Base>, fixed: bool) -> Self {
        if fixed {
            CellState::FixedValue(value)
        } else {
            CellState::Value(value)
        }
    }

    pub(super) fn with_candidates(candidates: Candidates<Base>) -> Self {
        CellState::Candidates(candidates)
    }

    pub(super) fn has_value(&self) -> bool {
        #[allow(clippy::match_same_arms)]
        match self {
            CellState::Value(_) => true,
            CellState::FixedValue(_) => true,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn has_unfixed_value(&self) -> bool {
        #[allow(clippy::match_same_arms)]
        match self {
            CellState::Value(_) => true,
            CellState::FixedValue(_) => false,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn has_fixed_value(&self) -> bool {
        #[allow(clippy::match_same_arms)]
        match self {
            CellState::Value(_) => false,
            CellState::FixedValue(_) => true,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn has_candidates(&self) -> bool {
        #[allow(clippy::match_same_arms)]
        match self {
            CellState::Value(_) => false,
            CellState::FixedValue(_) => false,
            CellState::Candidates(_) => true,
        }
    }

    pub(super) fn fix(&mut self) {
        *self = match *self {
            CellState::Value(value) | CellState::FixedValue(value) => CellState::FixedValue(value),
            CellState::Candidates(_) => panic!("Candidates can't be fixed: {self}"),
        };
    }

    pub(super) fn unfix(&mut self) {
        *self = match self {
            CellState::Value(value) | CellState::FixedValue(value) => CellState::Value(*value),
            CellState::Candidates(candidates) => CellState::Candidates(*candidates),
        };
    }

    pub(super) fn value(&self) -> Option<Value<Base>> {
        match self {
            &CellState::Value(value) | &CellState::FixedValue(value) => Some(value),
            CellState::Candidates(_) => None,
        }
    }

    pub(super) fn candidates(&self) -> Option<Candidates<Base>> {
        match self {
            CellState::Candidates(candidates) => Some(*candidates),
            _ => None,
        }
    }

    pub(super) fn delete(&mut self) {
        self.assert_unfixed();

        *self = Self::new();
    }

    pub(super) fn set_value(&mut self, value: Value<Base>) {
        self.assert_unfixed();

        *self = Self::with_value(value, false);
    }

    pub(super) fn set_or_toggle_value(&mut self, value: Value<Base>) -> bool {
        self.assert_unfixed();

        match self {
            CellState::Value(current_value) => {
                if current_value == &value {
                    self.delete();
                    false
                } else {
                    self.set_value(value);
                    true
                }
            }
            CellState::Candidates(_) => {
                self.set_value(value);
                true
            }
            CellState::FixedValue(_) => unreachable!(),
        }
    }

    pub(super) fn set_candidates(&mut self, candidates: Candidates<Base>) {
        self.assert_unfixed();

        *self = Self::with_candidates(candidates);
    }

    pub(super) fn toggle_candidate(&mut self, candidate: Value<Base>) {
        self.assert_unfixed();

        match self {
            CellState::Candidates(candidates) => {
                candidates.toggle(candidate);
            }
            CellState::Value(_) => {
                *self = Self::with_candidates(Candidates::with_single(candidate));
            }
            CellState::FixedValue(_) => unreachable!(),
        }
    }

    pub(super) fn set_candidate(&mut self, candidate: Value<Base>) {
        self.assert_unfixed();

        match self {
            CellState::Candidates(candidates) => candidates.set(candidate, true),
            CellState::Value(_) => {}
            CellState::FixedValue(_) => unreachable!(),
        };
    }
    pub(super) fn delete_candidate(&mut self, candidate: Value<Base>) {
        self.assert_unfixed();

        match self {
            CellState::Candidates(candidates) => candidates.delete(candidate),
            CellState::Value(_) => {}
            CellState::FixedValue(_) => unreachable!(),
        };
    }
}

/// Private helpers
impl<Base: SudokuBase> CellState<Base> {
    fn assert_unfixed(&self) {
        if let CellState::FixedValue(_) = self {
            panic!("Fixed cell can't be modified: {self}")
        }
    }
}

impl<Base> Display for CellState<Base>
where
    Base: SudokuBase,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(value) = self.value() {
            write!(f, "{value}")
        } else {
            write!(f, "0")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::base::consts::*;

    use super::*;

    struct Setup {
        value: Value<Base2>,
        candidates: Candidates<Base2>,
        candidates_2: Candidates<Base2>,
        cell_state_unfixed_value: CellState<Base2>,
        cell_state_unfixed_value_2: CellState<Base2>,
        cell_state_fixed_value: CellState<Base2>,
        cell_state_candidates: CellState<Base2>,
        cell_state_candidates_2: CellState<Base2>,
        cell_state_empty_candidates: CellState<Base2>,
    }

    fn setup() -> Setup {
        let value = Value::try_from(1).unwrap();
        let different_value = Value::try_from(2).unwrap();
        let candidates = Candidates::with_single(value);
        let candidates_2 = Candidates::with_single(different_value);
        let cell_state_unfixed_value = CellState::<Base2>::with_value(value, false);
        let cell_state_unfixed_value_2 = CellState::<Base2>::with_value(different_value, false);
        let cell_state_fixed_value = CellState::<Base2>::with_value(value, true);
        let cell_state_candidates = CellState::<Base2>::with_candidates(candidates);
        let cell_state_candidates_2 = CellState::<Base2>::with_candidates(candidates_2);
        let cell_state_empty_candidates = CellState::<Base2>::new();

        Setup {
            value,
            candidates,
            candidates_2,
            cell_state_unfixed_value,
            cell_state_unfixed_value_2,
            cell_state_fixed_value,
            cell_state_candidates,
            cell_state_candidates_2,
            cell_state_empty_candidates,
        }
    }

    #[test]
    fn test_cell_state_size() {
        assert_eq!(
            vec![
                size_of::<CellState<Base2>>(),
                size_of::<CellState<Base3>>(),
                size_of::<CellState<Base4>>(),
                size_of::<CellState<Base5>>()
            ],
            vec![2, 4, 4, 8]
        );
    }

    #[test]
    fn test_new() {
        assert_eq!(
            CellState::<Base2>::new(),
            CellState::with_candidates(Candidates::new())
        );
    }

    #[test]
    fn test_with_value() {
        let Setup {
            value,
            cell_state_fixed_value,
            cell_state_unfixed_value,
            ..
        } = setup();
        assert_eq!(cell_state_fixed_value.value(), Some(value));
        assert_eq!(cell_state_unfixed_value.value(), Some(value));
    }

    #[test]
    fn test_with_candidates() {
        let Setup {
            candidates,
            cell_state_candidates,
            ..
        } = setup();
        assert_eq!(cell_state_candidates.candidates(), Some(candidates));
    }

    #[test]
    fn test_has() {
        let Setup {
            cell_state_unfixed_value,
            cell_state_fixed_value,
            cell_state_candidates,
            ..
        } = setup();

        assert!(cell_state_unfixed_value.has_value());
        assert!(cell_state_unfixed_value.has_unfixed_value());
        assert!(!cell_state_unfixed_value.has_fixed_value());
        assert!(!cell_state_unfixed_value.has_candidates());

        assert!(cell_state_fixed_value.has_value());
        assert!(!cell_state_fixed_value.has_unfixed_value());
        assert!(cell_state_fixed_value.has_fixed_value());
        assert!(!cell_state_fixed_value.has_candidates());

        assert!(!cell_state_candidates.has_value());
        assert!(!cell_state_candidates.has_unfixed_value());
        assert!(!cell_state_candidates.has_fixed_value());
        assert!(cell_state_candidates.has_candidates());
    }

    #[test]
    fn test_fixing() {
        let Setup {
            cell_state_unfixed_value: mut cell_state,
            ..
        } = setup();

        assert!(cell_state.has_unfixed_value());
        cell_state.unfix();
        assert!(cell_state.has_unfixed_value());
        cell_state.fix();
        assert!(cell_state.has_fixed_value());
        cell_state.fix();
        assert!(cell_state.has_fixed_value());
        cell_state.unfix();
        assert!(cell_state.has_unfixed_value());
    }

    #[test]
    #[should_panic(expected = "Candidates can't be fixed")]
    fn test_fix_panic() {
        setup().cell_state_empty_candidates.fix();
    }

    #[test]
    fn test_delete() {
        let Setup {
            mut cell_state_unfixed_value,
            mut cell_state_candidates,
            cell_state_empty_candidates,
            ..
        } = setup();

        cell_state_unfixed_value.delete();
        cell_state_candidates.delete();

        assert_eq!(cell_state_unfixed_value, cell_state_empty_candidates);
        assert_eq!(cell_state_candidates, cell_state_empty_candidates);
    }

    #[test]
    #[should_panic(expected = "Fixed cell can't be modified")]
    fn test_delete_panic() {
        setup().cell_state_fixed_value.delete();
    }

    #[test]
    fn test_set_value() {
        let Setup {
            value,
            mut cell_state_unfixed_value,
            mut cell_state_candidates,
            ..
        } = setup();

        cell_state_unfixed_value.set_value(value);
        cell_state_candidates.set_value(value);

        assert_eq!(cell_state_unfixed_value.value(), Some(value));
        assert_eq!(cell_state_candidates.value(), Some(value));
    }

    #[test]
    #[should_panic(expected = "Fixed cell can't be modified")]
    fn test_set_value_panic() {
        let Setup {
            mut cell_state_fixed_value,
            value,
            ..
        } = setup();
        cell_state_fixed_value.set_value(value);
    }

    #[test]
    fn test_set_or_toggle_value() {
        let Setup {
            value,
            mut cell_state_unfixed_value,
            mut cell_state_unfixed_value_2,
            mut cell_state_candidates,
            cell_state_empty_candidates,
            ..
        } = setup();

        cell_state_unfixed_value.set_or_toggle_value(value);
        cell_state_unfixed_value_2.set_or_toggle_value(value);
        cell_state_candidates.set_or_toggle_value(value);

        assert_eq!(cell_state_unfixed_value, cell_state_empty_candidates);
        assert_eq!(cell_state_unfixed_value_2.value(), Some(value));
        assert_eq!(cell_state_candidates.value(), Some(value));
    }

    #[test]
    #[should_panic(expected = "Fixed cell can't be modified")]
    fn test_set_or_toggle_value_panic() {
        let Setup {
            mut cell_state_fixed_value,
            value,
            ..
        } = setup();
        cell_state_fixed_value.set_or_toggle_value(value);
    }

    #[test]
    fn test_set_candidates() {
        let Setup {
            candidates,
            mut cell_state_unfixed_value,
            mut cell_state_candidates,
            ..
        } = setup();

        cell_state_unfixed_value.set_candidates(candidates);
        cell_state_candidates.set_candidates(candidates);

        assert_eq!(cell_state_unfixed_value.candidates(), Some(candidates));
        assert_eq!(cell_state_candidates.candidates(), Some(candidates));
    }

    #[test]
    #[should_panic(expected = "Fixed cell can't be modified")]
    fn test_set_candidates_panic() {
        let Setup {
            mut cell_state_fixed_value,
            candidates,
            ..
        } = setup();
        cell_state_fixed_value.set_candidates(candidates);
    }

    #[test]
    fn test_toggle_candidate() {
        let Setup {
            value,
            candidates,
            candidates_2,
            cell_state_empty_candidates,
            mut cell_state_unfixed_value,
            mut cell_state_candidates,
            mut cell_state_candidates_2,
            ..
        } = setup();

        cell_state_unfixed_value.toggle_candidate(value);
        cell_state_candidates.toggle_candidate(value);
        cell_state_candidates_2.toggle_candidate(value);

        assert_eq!(cell_state_unfixed_value.candidates(), Some(candidates));
        assert_eq!(cell_state_candidates, cell_state_empty_candidates);
        assert_eq!(
            cell_state_candidates_2.candidates(),
            Some(candidates.union(candidates_2))
        );
    }

    #[test]
    #[should_panic(expected = "Fixed cell can't be modified")]
    fn test_toggle_candidate_panic() {
        let Setup {
            mut cell_state_fixed_value,
            value,
            ..
        } = setup();
        cell_state_fixed_value.toggle_candidate(value);
    }

    #[test]
    fn test_set_candidate() {
        let Setup {
            value,
            candidates,
            candidates_2,
            mut cell_state_unfixed_value,
            mut cell_state_candidates,
            mut cell_state_candidates_2,
            ..
        } = setup();

        cell_state_unfixed_value.set_candidate(value);
        cell_state_candidates.set_candidate(value);
        cell_state_candidates_2.set_candidate(value);

        assert_eq!(cell_state_unfixed_value.value(), Some(value));
        assert_eq!(cell_state_candidates.candidates(), Some(candidates));
        assert_eq!(
            cell_state_candidates_2.candidates(),
            Some(candidates.union(candidates_2))
        );
    }

    #[test]
    #[should_panic(expected = "Fixed cell can't be modified")]
    fn test_set_candidate_panic() {
        let Setup {
            mut cell_state_fixed_value,
            value,
            ..
        } = setup();
        cell_state_fixed_value.set_candidate(value);
    }

    #[test]
    fn test_delete_candidate() {
        let Setup {
            value,
            candidates_2,
            cell_state_empty_candidates,
            mut cell_state_unfixed_value,
            mut cell_state_candidates,
            mut cell_state_candidates_2,
            ..
        } = setup();

        cell_state_unfixed_value.delete_candidate(value);
        cell_state_candidates.delete_candidate(value);
        cell_state_candidates_2.delete_candidate(value);

        assert_eq!(cell_state_unfixed_value.value(), Some(value));
        assert_eq!(cell_state_candidates, cell_state_empty_candidates);
        assert_eq!(cell_state_candidates_2.candidates(), Some(candidates_2));
    }

    #[test]
    #[should_panic(expected = "Fixed cell can't be modified")]
    fn test_delete_candidate_panic() {
        let Setup {
            mut cell_state_fixed_value,
            value,
            ..
        } = setup();
        cell_state_fixed_value.delete_candidate(value);
    }
}
