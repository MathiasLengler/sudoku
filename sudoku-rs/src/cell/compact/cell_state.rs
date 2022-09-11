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
        Self::with_candidates(Candidates::new())
    }
}

impl<Base: SudokuBase> CellState<Base> {
    pub(super) fn new() -> Self {
        Default::default()
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
        match self {
            CellState::Value(_) => true,
            CellState::FixedValue(_) => true,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn has_unfixed_value(&self) -> bool {
        match self {
            CellState::Value(_) => true,
            CellState::FixedValue(_) => false,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn has_fixed_value(&self) -> bool {
        match self {
            CellState::Value(_) => false,
            CellState::FixedValue(_) => true,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn has_candidates(&self) -> bool {
        match self {
            CellState::Value(_) => false,
            CellState::FixedValue(_) => false,
            CellState::Candidates(_) => true,
        }
    }

    pub(super) fn fix(&mut self) {
        *self = match *self {
            CellState::Value(value) => CellState::FixedValue(value),
            CellState::FixedValue(value) => CellState::FixedValue(value),
            CellState::Candidates(_) => panic!("Candidates can't be fixed: {}", self),
        };
    }

    pub(super) fn unfix(&mut self) {
        *self = match self {
            CellState::Value(value) => CellState::Value(*value),
            CellState::FixedValue(value) => CellState::Value(*value),
            CellState::Candidates(ref candidates) => CellState::Candidates(candidates.clone()),
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
            CellState::Candidates(candidates) => Some(candidates.clone()),
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
            _ => unreachable!(),
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
                *self = Self::with_candidates(Candidates::single(candidate));
            }
            _ => unreachable!(),
        }
    }

    pub(super) fn delete_candidate(&mut self, candidate: Value<Base>) {
        self.assert_unfixed();

        match self {
            CellState::Candidates(candidates) => candidates.delete(candidate),
            CellState::Value(_) => {}
            _ => unreachable!(),
        };
    }
}

/// Private helpers
impl<Base: SudokuBase> CellState<Base> {
    fn assert_unfixed(&self) {
        if let CellState::FixedValue(_) = self {
            panic!("Fixed cell can't be modified: {}", self)
        }
    }
}

impl<Base> Display for CellState<Base>
where
    Base: SudokuBase,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            if let Some(value) = self.value() {
                value.to_string()
            } else {
                "0".to_string()
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::base::consts::*;

    use super::*;

    #[ignore]
    #[test]
    fn test_cell_state_size() {
        assert_eq!(
            vec![
                size_of::<CellState<U1>>(),
                size_of::<CellState<U2>>(),
                size_of::<CellState<U3>>(),
                size_of::<CellState<U4>>(),
                size_of::<CellState<U5>>()
            ],
            vec![2, 2, 3, 3, 5,]
        )
    }
}
