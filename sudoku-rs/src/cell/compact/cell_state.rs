use std::cmp::Eq;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::mem::replace;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::cell::view::CellView;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub(super) enum CellState<Base: SudokuBase> {
    Value(Value<Base>),
    FixedValue(Value<Base>),
    Candidates(Candidates<Base>),
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

    pub(super) fn view(&self) -> CellView {
        match self {
            CellState::Value(value) => CellView::Value {
                value: value.into_u8(),
                fixed: false,
            },
            CellState::FixedValue(value) => CellView::Value {
                value: value.into_u8(),
                fixed: true,
            },
            CellState::Candidates(candidates) => CellView::Candidates {
                candidates: candidates.to_vec_u8(),
            },
        }
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
        replace(
            self,
            match self {
                &mut CellState::Value(value) => CellState::FixedValue(value),
                &mut CellState::FixedValue(value) => CellState::FixedValue(value),
                &mut CellState::Candidates(_) => panic!("Candidates can't be fixed: {}", self),
            },
        );
    }

    pub(super) fn unfix(&mut self) {
        replace(
            self,
            match &*self {
                &CellState::Value(value) => CellState::Value(value),
                &CellState::FixedValue(value) => CellState::Value(value),
                CellState::Candidates(candidates) => CellState::Candidates(candidates.clone()),
            },
        );
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

        replace(self, Self::new());
    }

    pub(super) fn set_value(&mut self, value: Value<Base>) {
        self.assert_unfixed();

        replace(self, Self::with_value(value, false));
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

        replace(self, Self::with_candidates(candidates));
    }

    pub(super) fn toggle_candidate(&mut self, candidate: Value<Base>) {
        self.assert_unfixed();

        match self {
            CellState::Candidates(candidates) => {
                candidates.toggle(candidate);
            }
            CellState::Value(_) => {
                // TODO: optimize with Candidate::single
                replace(self, Self::with_candidates(vec![candidate].into()));
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
        match self {
            // TODO: bail instead of panic
            //
            CellState::FixedValue(_) => panic!("Fixed cell can't be modified: {}", self),
            _ => {}
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
                "_".to_string()
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use std::num::NonZeroU8;

    use typenum::consts::*;

    use crate::error::Result;

    use super::*;

    #[test]
    fn test_import_value() -> Result<()> {
        let value = CellState::<U3>::import_value(9)?;
        assert_eq!(value, NonZeroU8::new(9).unwrap());

        let value = CellState::<U3>::import_value(0);
        assert!(value.is_err());

        let value = CellState::<U3>::import_value(10);
        assert!(value.is_err());

        Ok(())
    }

    #[test]
    fn test_cell_state_size() {
        assert_eq!(size_of::<CellState<U1>>(), 2);
        assert_eq!(size_of::<CellState<U2>>(), 2);
        assert_eq!(size_of::<CellState<U3>>(), 3);
        assert_eq!(size_of::<CellState<U4>>(), 3);
        assert_eq!(size_of::<CellState<U5>>(), 5);
    }
}
