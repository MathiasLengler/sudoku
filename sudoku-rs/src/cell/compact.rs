use std::cmp::Eq;
use std::convert::TryInto;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::mem::{align_of, replace, size_of, swap};
use std::num::NonZeroU8;
use std::ops::*;

use bitvec::prelude::*;
use failure::_core::intrinsics::write_bytes;
use fixedbitset::FixedBitSet;
use generic_array::{ArrayLength, GenericArray};
use typenum::{assert_type, bit::B1, consts::*, op, Prod, Quot, Sub1, Sum, Unsigned};

use sudoku_base::SudokuBase;

use crate::cell::view::CellView;
use crate::cell::SudokuCell;

mod sudoku_base;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub enum Cell<Base: SudokuBase> {
    Value(NonZeroU8),
    FixedValue(NonZeroU8),
    Candidates(GenericArray<u8, Base::CandidatesCapacity>),
}

impl<Base: SudokuBase> Cell<Base> {
    fn new() -> Self {
        Self::with_candidates(std::iter::empty())
    }

    fn with_value(value: u8, fixed: bool) -> Self {
        if value == 0 {
            Self::new()
        } else {
            let value = Self::import_value(value);
            if fixed {
                Cell::FixedValue(value)
            } else {
                Cell::Value(value)
            }
        }
    }

    fn with_candidates<I>(candidates: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        Cell::Candidates(Self::import_candidates(candidates))
    }

    fn view(&self) -> CellView {
        // TODO: remove extra conversions
        match self {
            Cell::Value(value) => CellView::Value {
                // TODO: fixed
                value: Self::export_value(*value).into(),
            },
            Cell::FixedValue(value) => CellView::Value {
                // TODO: fixed
                value: Self::export_value(*value).into(),
            },
            Cell::Candidates(candidates) => CellView::Candidates {
                candidates: Self::export_candidates(candidates)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
        }
    }
    fn is_value(&self) -> bool {
        match self {
            Cell::Value(_) => true,
            Cell::FixedValue(_) => true,
            Cell::Candidates(_) => false,
        }
    }
    fn is_unfixed_value(&self) -> bool {
        match self {
            Cell::Value(_) => true,
            Cell::FixedValue(_) => false,
            Cell::Candidates(_) => false,
        }
    }
    fn is_fixed_value(&self) -> bool {
        match self {
            Cell::Value(_) => false,
            Cell::FixedValue(_) => true,
            Cell::Candidates(_) => false,
        }
    }
    fn is_candidates(&self) -> bool {
        match self {
            Cell::Value(_) => false,
            Cell::FixedValue(_) => false,
            Cell::Candidates(_) => true,
        }
    }

    fn fix(&mut self) {
        replace(
            self,
            match self {
                &mut Cell::Value(value) => Cell::FixedValue(value),
                &mut Cell::FixedValue(value) => Cell::FixedValue(value),
                &mut Cell::Candidates(_) => panic!("Candidates can't be fixed: {}", self),
            },
        );
    }

    fn unfix(&mut self) {
        replace(
            self,
            match &*self {
                &Cell::Value(value) => Cell::Value(value),
                &Cell::FixedValue(value) => Cell::Value(value),
                Cell::Candidates(candidates) => Cell::Candidates(candidates.clone()),
            },
        );
    }

    fn value(&self) -> Option<u8> {
        match self {
            &Cell::Value(value) => Some(Self::export_value(value)),
            &Cell::FixedValue(value) => Some(Self::export_value(value)),
            Cell::Candidates(_) => None,
        }
    }

    fn candidates(&self) -> Option<Vec<u8>> {
        match self {
            Cell::Candidates(candidates) => Some(Self::export_candidates(candidates)),
            _ => None,
        }
    }

    fn delete(&mut self) -> Self {
        self.assert_unfixed();

        replace(self, Self::new())
    }

    fn set_value(&mut self, value: u8) {
        self.assert_unfixed();

        replace(self, Self::with_value(value, false));
    }

    fn set_or_toggle_value(&mut self, value: u8) -> bool {
        self.assert_unfixed();

        match self {
            Cell::Value(current_value) => {
                if Self::export_value(*current_value) == value {
                    self.delete();
                    false
                } else {
                    self.set_value(value);
                    true
                }
            }
            Cell::Candidates(_) => {
                self.set_value(value);
                true
            }
            _ => unreachable!(),
        }
    }

    fn set_candidates<I>(&mut self, candidates: I)
    where
        I: IntoIterator<Item = u8>,
    {
        self.assert_unfixed();

        replace(self, Self::with_candidates(candidates));
    }

    fn toggle_candidate(&mut self, candidate: u8) {
        self.assert_unfixed();

        let imported_candidate = Self::import_candidate(candidate);

        match self {
            Cell::Candidates(candidates) => {
                let bs = Self::candidates_as_mut_bitslice(candidates);

                bs.set(imported_candidate, !bs[imported_candidate]);
            }
            Cell::Value(_) => {
                replace(self, Self::with_candidates(std::iter::once(candidate)));
            }
            _ => unreachable!(),
        }
    }

    fn delete_candidate(&mut self, candidate: u8) {
        self.assert_unfixed();

        let imported_candidate = Self::import_candidate(candidate);

        match self {
            Cell::Candidates(candidates) => {
                let bs = Self::candidates_as_mut_bitslice(candidates);

                bs.set(imported_candidate, false);
            }
            Cell::Value(_) => {}
            _ => unreachable!(),
        }
    }
}

/// Private helpers
impl<Base: SudokuBase> Cell<Base> {
    fn assert_unfixed(&self) {
        match self {
            Cell::FixedValue(_) => panic!("Fixed cell can't be modified: {}", self),
            _ => {}
        }
    }
}

// TODO: Update trait SudokuCell and impl
//  alternative: remove SudokuCell (leaky)

/// Conversion Helpers
impl<Base: SudokuBase> Cell<Base> {
    fn candidates_as_bitslice(candidates: &[u8]) -> &BitSlice<LittleEndian> {
        candidates.as_bitslice::<LittleEndian>()
    }

    fn candidates_as_mut_bitslice(candidates: &mut [u8]) -> &mut BitSlice<LittleEndian> {
        candidates.as_mut_bitslice::<LittleEndian>()
    }

    fn import_candidates<I: IntoIterator<Item = u8>>(
        candidates: I,
    ) -> GenericArray<u8, Base::CandidatesCapacity> {
        let mut arr = GenericArray::<u8, Base::CandidatesCapacity>::default();

        let bs = Self::candidates_as_mut_bitslice(&mut arr);

        for candidate in candidates {
            bs.set(Self::import_candidate(candidate), true);
        }

        debug_assert!(bs[Base::MaxValue::to_usize()..].not_any());

        arr
    }

    fn import_candidate(candidate: u8) -> usize {
        assert_ne!(candidate, 0);
        assert!(candidate <= Base::MaxValue::to_u8());

        (candidate - 1).into()
    }

    fn import_value(value: u8) -> NonZeroU8 {
        assert!(value <= Base::MaxValue::to_u8());

        let value = NonZeroU8::new(value).expect("Value can't be 0");

        value
    }

    fn export_value(value: NonZeroU8) -> u8 {
        value.get()
    }

    fn export_candidates(candidates: &[u8]) -> Vec<u8> {
        let bs = Self::candidates_as_bitslice(candidates);

        bs.iter()
            .enumerate()
            .filter_map(|(i, is_set)| {
                if is_set {
                    Some(Self::export_candidate(i))
                } else {
                    None
                }
            })
            .collect()
    }

    fn export_candidate(candidate: usize) -> u8 {
        (candidate + 1).try_into().unwrap()
    }
}

impl<Base> Display for Cell<Base>
where
    Base: SudokuBase,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
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
    use crate::cell::Cell as OldCell;

    use super::*;

    #[test]
    fn test_import_candidates() {
        use generic_array::arr;

        let array = Cell::<U3>::import_candidates(vec![1, 2, 4, 8, 9]);

        assert_eq!(array, arr![u8; 0b1000_1011, 0b0000_0001]);

        let array = Cell::<U3>::import_candidates(std::iter::empty());

        assert_eq!(array, arr![u8; 0, 0]);
    }

    #[test]
    #[should_panic]
    fn test_import_candidates_panic_max() {
        Cell::<U3>::import_candidates(vec![10]);
    }

    #[test]
    #[should_panic]
    fn test_import_candidates_panic_zero() {
        Cell::<U3>::import_candidates(vec![0]);
    }

    #[test]
    fn test_import_value() {
        let value = Cell::<U3>::import_value(9);

        assert_eq!(value, NonZeroU8::new(9).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_import_value_panic_zero() {
        Cell::<U3>::import_value(0);
    }

    #[test]
    fn test_compact_cell_size() {
        type Base = U3;
        dbg!(size_of::<Cell<Base>>());
        dbg!(align_of::<Cell<Base>>());
        dbg!(size_of::<[Cell<Base>; 2]>());
        dbg!(size_of::<OldCell>());
        dbg!(size_of::<FixedBitSet>());
        dbg!(size_of::<Vec<u32>>());
        dbg!(size_of::<usize>());

        const TRANSMUTE_SIZE: usize = 3;

        //        let cell = CompactCell::<Base>::Value(NonZeroU8::new(255).unwrap());
        //        println!("{:02X?}", unsafe {
        //            std::mem::transmute::<_, [u8; TRANSMUTE_SIZE]>(cell)
        //        });
        //        let cell = CompactCell::<Base>::FixedValue(NonZeroU8::new(0xab).unwrap());
        //        println!("{:02X?}", unsafe {
        //            std::mem::transmute::<_, [u8; TRANSMUTE_SIZE]>(cell)
        //        });
        //        let cell = CompactCell::<Base>::Candidates([0b1010_0101; 2].into());
        //        println!("{:02X?}", unsafe {
        //            std::mem::transmute::<_, [u8; TRANSMUTE_SIZE]>(cell)
        //        });
    }
}
