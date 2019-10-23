use std::convert::TryInto;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

use bitvec::prelude::*;
use generic_array::GenericArray;
use typenum::Unsigned;

use crate::base::{ArrayElement, SudokuBase};

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug, Default)]
pub struct Candidates<Base: SudokuBase> {
    arr: GenericArray<ArrayElement, Base::CandidatesCapacity>,
}

impl<Base: SudokuBase> Candidates<Base> {
    pub fn all() -> Self {
        let mut this = Self::default();

        let bits = this.as_mut_bits();

        bits[0..Base::MaxValue::to_usize()].set_all(true);

        this.debug_assert();

        this
    }

    pub fn toggle(&mut self, candidate: u8) {
        let imported_candidate = Self::import(candidate);

        let bits = self.as_mut_bits();

        bits.set(imported_candidate, !bits[imported_candidate]);

        self.debug_assert();
    }

    pub fn delete(&mut self, candidate: u8) {
        let imported_candidate = Self::import(candidate);

        let bits = self.as_mut_bits();

        bits.set(imported_candidate, false);

        self.debug_assert();
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
    fn as_bits(&self) -> &BitSlice<LittleEndian, ArrayElement> {
        self.arr.as_bitslice::<LittleEndian>()
    }

    fn as_mut_bits(&mut self) -> &mut BitSlice<LittleEndian, ArrayElement> {
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
        debug_assert!({
            let bits = self.as_bits();
            bits[Base::MaxValue::to_usize()..].not_any()
        });
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

impl<Base: SudokuBase, I: IntoIterator<Item = u8>> From<I> for Candidates<Base> {
    fn from(into_iter: I) -> Self {
        Self::from_iter(into_iter)
    }
}

impl<Base: SudokuBase> Display for Candidates<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use typenum::consts::*;

    use super::*;

    #[test]
    fn test_from_u8_iter() {
        use generic_array::arr;

        let vec_candidates = vec![1, 2, 4, 8, 9];

        let candidates = Candidates::<U3>::from_iter(vec_candidates.clone());

        assert_eq!(candidates.to_vec(), vec_candidates);

        let candidates = Candidates::<U3>::from_iter(std::iter::empty());

        assert_eq!(candidates.to_vec(), vec![]);
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
