use std::cmp::Eq;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::mem::{align_of, size_of};
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

// TODO: Copy?
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub enum Cell<Base: SudokuBase> {
    Value(NonZeroU8),
    FixedValue(NonZeroU8),
    Candidates(GenericArray<u8, Base::CandidatesCapacity>),
}

impl<Base: SudokuBase> Cell<Base> {
    fn new() -> Self {
        Self::new_with_candidates(std::iter::empty())
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

    fn new_with_candidates<I>(candidates: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        Cell::Candidates(Self::import_candidates(candidates))
    }

    fn view(&self) -> CellView {
        unimplemented!()
    }

    fn value(&self) -> Option<u8> {
        unimplemented!()
    }

    fn candidates(&self) -> Option<Vec<u8>> {
        unimplemented!()
    }

    fn delete(&mut self) -> Self {
        unimplemented!()
    }

    fn set_value(&mut self, value: u8) {
        unimplemented!()
    }

    fn set_or_toggle_value(&mut self, value: u8) -> bool {
        unimplemented!()
    }

    fn set_candidates<I>(&mut self, candidates: I)
    where
        I: IntoIterator<Item = u8>,
    {
    }

    fn toggle_candidate(&mut self, candidate: u8) {
        unimplemented!()
    }

    fn delete_candidate(&mut self, candidate: u8) {
        unimplemented!()
    }
}

// TODO: Update trait SudokuCell and impl
//  alternative: remove SudokuCell (leaky)

impl<Base> Cell<Base>
where
    Base: SudokuBase,
{
    fn import_candidates<I: IntoIterator<Item = u8>>(
        candidates: I,
    ) -> GenericArray<u8, Base::CandidatesCapacity> {
        let mut arr = GenericArray::<u8, Base::CandidatesCapacity>::default();

        let bs = arr.as_mut_bitslice::<LittleEndian>();

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

    // TODO:
    //  export_value
    //  export_candidates
    //  export_candidate
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
