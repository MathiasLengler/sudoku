use std::convert::TryInto;
use std::iter::FromIterator;

use bitvec::prelude::*;
use generic_array::GenericArray;
use typenum::Unsigned;

use crate::base::SudokuBase;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug, Default)]
pub struct Candidates<Base: SudokuBase> {
    arr: GenericArray<u8, Base::CandidatesCapacity>,
}

impl<Base: SudokuBase> Candidates<Base> {
    pub fn toggle(&mut self, candidate: u8) {
        let imported_candidate = Self::import(candidate);

        let bits = self.as_mut_bits();

        bits.set(imported_candidate, !bits[imported_candidate]);
    }

    pub fn delete(&mut self, candidate: u8) {
        let imported_candidate = Self::import(candidate);

        let bits = self.as_mut_bits();

        bits.set(imported_candidate, false);
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let bits = self.as_bits();

        bits.iter()
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
}

impl<Base: SudokuBase> Candidates<Base> {
    fn as_bits(&self) -> &BitSlice<LittleEndian> {
        self.arr.as_bitslice::<LittleEndian>()
    }

    fn as_mut_bits(&mut self) -> &mut BitSlice<LittleEndian> {
        self.arr.as_mut_bitslice::<LittleEndian>()
    }

    fn import(candidate: u8) -> usize {
        assert_ne!(candidate, 0);
        assert!(candidate <= Base::MaxValue::to_u8());

        (candidate - 1).into()
    }

    fn export_candidate(candidate: usize) -> u8 {
        (candidate + 1).try_into().unwrap()
    }

    fn debug_assert(&self) {
        let bits = self.as_bits();
        debug_assert!(bits[Base::MaxValue::to_usize()..].not_any());
    }
}

impl<Base: SudokuBase> FromIterator<u8> for Candidates<Base> {
    fn from_iter<T: IntoIterator<Item = u8>>(candidates: T) -> Self {
        let mut this = Self::default();

        let bits = this.as_mut_bits();

        for candidate in candidates {
            bits.set(Self::import(candidate), true);
        }

        this.debug_assert();

        this
    }
}

#[cfg(test)]
mod tests {
    use typenum::consts::*;

    use super::*;

    #[test]
    fn test_from_u8_iter() {
        use generic_array::arr;

        let candidates = Candidates::<U3>::from_iter(vec![1, 2, 4, 8, 9]);

        assert_eq!(candidates.arr, arr![u8; 0b1000_1011, 0b0000_0001]);

        let candidates = Candidates::<U3>::from_iter(std::iter::empty());

        assert_eq!(candidates.arr, arr![u8; 0, 0]);
    }

    #[test]
    #[should_panic]
    fn test_from_u8_iter_panic_max() {
        Candidates::<U3>::from_iter(vec![10]);
    }

    #[test]
    #[should_panic]
    fn test_from_u8_iter_panic_zero() {
        Candidates::<U3>::from_iter(vec![0]);
    }
}
