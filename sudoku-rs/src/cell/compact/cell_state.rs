use std::cmp::Eq;
use std::convert::TryInto;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::mem::replace;
use std::num::NonZeroU8;

use bitvec::prelude::*;
use generic_array::GenericArray;
use typenum::Unsigned;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::view::CellView;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub(super) enum CellState<Base: SudokuBase> {
    Value(NonZeroU8),
    FixedValue(NonZeroU8),
    Candidates(Candidates<Base>),
}

impl<Base: SudokuBase> CellState<Base> {
    pub(super) fn new() -> Self {
        Self::with_candidates(std::iter::empty())
    }

    pub(super) fn with_value(value: u8, fixed: bool) -> Self {
        if value == 0 {
            Self::new()
        } else {
            let value = Self::import_value(value);
            if fixed {
                CellState::FixedValue(value)
            } else {
                CellState::Value(value)
            }
        }
    }

    pub(super) fn with_candidates<I>(candidates: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        CellState::Candidates(candidates.into_iter().collect())
    }

    pub(super) fn view(&self) -> CellView {
        // TODO: remove extra conversions
        match self {
            CellState::Value(value) => CellView::Value {
                value: Self::export_value(*value),
                fixed: false,
            },
            CellState::FixedValue(value) => CellView::Value {
                value: Self::export_value(*value),
                fixed: true,
            },
            CellState::Candidates(candidates) => CellView::Candidates {
                candidates: candidates.to_vec(),
            },
        }
    }
    pub(super) fn is_value(&self) -> bool {
        match self {
            CellState::Value(_) => true,
            CellState::FixedValue(_) => true,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn is_unfixed_value(&self) -> bool {
        match self {
            CellState::Value(_) => true,
            CellState::FixedValue(_) => false,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn is_fixed_value(&self) -> bool {
        match self {
            CellState::Value(_) => false,
            CellState::FixedValue(_) => true,
            CellState::Candidates(_) => false,
        }
    }
    pub(super) fn is_candidates(&self) -> bool {
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

    pub(super) fn value(&self) -> Option<u8> {
        match self {
            &CellState::Value(value) => Some(Self::export_value(value)),
            &CellState::FixedValue(value) => Some(Self::export_value(value)),
            CellState::Candidates(_) => None,
        }
    }

    // TODO: expose candidates directly
    pub(super) fn candidates(&self) -> Option<Vec<u8>> {
        match self {
            CellState::Candidates(candidates) => Some(candidates.to_vec()),
            _ => None,
        }
    }

    pub(super) fn delete(&mut self) {
        self.assert_unfixed();

        replace(self, Self::new());
    }

    pub(super) fn set_value(&mut self, value: u8) {
        self.assert_unfixed();

        replace(self, Self::with_value(value, false));
    }

    pub(super) fn set_or_toggle_value(&mut self, value: u8) -> bool {
        self.assert_unfixed();

        match self {
            CellState::Value(current_value) => {
                if Self::export_value(*current_value) == value {
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

    pub(super) fn set_candidates<I>(&mut self, candidates: I)
    where
        I: IntoIterator<Item = u8>,
    {
        self.assert_unfixed();

        replace(self, Self::with_candidates(candidates));
    }

    pub(super) fn toggle_candidate(&mut self, candidate: u8) {
        self.assert_unfixed();

        match self {
            CellState::Candidates(candidates) => {
                candidates.toggle(candidate);
            }
            CellState::Value(_) => {
                replace(self, Self::with_candidates(std::iter::once(candidate)));
            }
            _ => unreachable!(),
        }
    }

    pub(super) fn delete_candidate(&mut self, candidate: u8) {
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
            CellState::FixedValue(_) => panic!("Fixed cell can't be modified: {}", self),
            _ => {}
        }
    }
}

// TODO: Update trait SudokuCell and impl
//  alternative: remove SudokuCell (leaky)

/// Conversion Helpers
impl<Base: SudokuBase> CellState<Base> {
    fn import_value(value: u8) -> NonZeroU8 {
        assert!(value <= Base::MaxValue::to_u8());

        let value = NonZeroU8::new(value).expect("Value can't be 0");

        value
    }

    fn export_value(value: NonZeroU8) -> u8 {
        value.get()
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

    use typenum::consts::*;

    use super::*;

    #[test]
    fn test_import_value() {
        let value = CellState::<U3>::import_value(9);

        assert_eq!(value, NonZeroU8::new(9).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_import_value_panic_zero() {
        CellState::<U3>::import_value(0);
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
