use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use bitvec::prelude::*;
use bitvec::view::BitView;
use generic_array::GenericArray;
use typenum::Unsigned;

use crate::base::{ArrayElement, SudokuBase};
use crate::cell::compact::value::Value;
use crate::error::{Error, Result};

// TODO: bench bitvec overhead
//  - BitArray.as_bitslice
//  - bits.set vs bit twiddling
//  - Deref

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct Candidates<Base: SudokuBase> {
    arr: BitArray<Base::CandidatesArray>,
}

impl<Base: SudokuBase> Default for Candidates<Base> {
    fn default() -> Self {
        Self {
            arr: BitArray::new(Base::CandidatesArray::ZERO),
        }
    }
}

impl<Base: SudokuBase> Candidates<Base> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all() -> Self {
        let mut this = Self::default();

        let bits = this.as_mut_bits();

        bits[0..Base::MAX_VALUE as usize].fill(true);

        this.debug_assert();

        this
    }

    pub fn toggle(&mut self, candidate: Value<Base>) {
        let imported_candidate = Self::import(candidate);

        let bits = self.as_mut_bits();

        bits.set(imported_candidate, !bits[imported_candidate]);

        self.debug_assert();
    }

    pub fn set(&mut self, candidate: Value<Base>, value: bool) {
        let imported_candidate = Self::import(candidate);

        let bits = self.as_mut_bits();

        bits.set(imported_candidate, value);

        self.debug_assert();
    }

    pub fn delete(&mut self, candidate: Value<Base>) {
        let imported_candidate = Self::import(candidate);

        let bits = self.as_mut_bits();

        bits.set(imported_candidate, false);

        self.debug_assert();
    }

    pub fn iter(&self) -> impl Iterator<Item = Value<Base>> + '_ {
        let bits = self.as_bits();

        bits.iter_ones().map(|i| Self::export(i))
    }

    pub fn to_vec_u8(&self) -> Vec<u8> {
        self.iter().map(|value| value.into_u8()).collect()
    }

    pub fn to_vec_value(&self) -> Vec<Value<Base>> {
        self.iter().collect()
    }

    /// Optimization to allow multiple modifications to candidates without recreating `BitSlice` wrapper.
    pub fn as_mut(&mut self) -> CandidatesMut<'_, Base> {
        CandidatesMut {
            bits: self.as_mut_bits(),
            base: PhantomData::default(),
        }
    }
}

impl<Base: SudokuBase> Candidates<Base> {
    fn as_bits(&self) -> &BitSlice<<Base::CandidatesArray as BitView>::Store> {
        self.arr.as_bitslice()
    }

    fn as_mut_bits(&mut self) -> &mut BitSlice<<Base::CandidatesArray as BitView>::Store> {
        self.arr.as_mut_bitslice()
    }

    fn import(candidate: Value<Base>) -> usize {
        (candidate.into_u8() - 1).into()
    }

    fn export(candidate: usize) -> Value<Base> {
        u8::try_from(candidate + 1).unwrap().try_into().unwrap()
    }

    fn debug_assert(&self) {
        debug_assert!({
            let bits = self.as_bits();
            bits[Base::MAX_VALUE as usize..].not_any()
        });
    }
}

impl<Base: SudokuBase> From<Vec<Value<Base>>> for Candidates<Base> {
    fn from(candidates: Vec<Value<Base>>) -> Self {
        let mut this = Self::default();

        let bits = this.as_mut_bits();

        for candidate in candidates {
            bits.set(Self::import(candidate), true);
        }

        this.debug_assert();

        this
    }
}

impl<Base: SudokuBase> TryFrom<Vec<u8>> for Candidates<Base> {
    type Error = Error;

    fn try_from(candidates: Vec<u8>) -> Result<Self> {
        let mut this = Self::default();

        let bits = this.as_mut_bits();

        for candidate in candidates {
            bits.set(Self::import(candidate.try_into()?), true);
        }

        this.debug_assert();

        Ok(this)
    }
}

impl<Base: SudokuBase> Display for Candidates<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_vec_u8())
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct CandidatesMut<'a, Base: SudokuBase> {
    bits: &'a mut BitSlice<<Base::CandidatesArray as BitView>::Store>,
    base: PhantomData<Base>,
}

// TODO: move other methods / use internally?
//  could require CandidatesRef for shared mutability methods
impl<'a, Base: SudokuBase> CandidatesMut<'a, Base> {
    pub fn delete(&mut self, candidate: Value<Base>) {
        let imported_candidate = Candidates::<Base>::import(candidate);

        self.bits.set(imported_candidate, false);
    }

    pub fn set(&mut self, candidate: Value<Base>, value: bool) {
        let imported_candidate = Candidates::<Base>::import(candidate);

        self.bits.set(imported_candidate, value);
    }

    fn debug_assert(&self) {
        debug_assert!(self.bits[Base::MAX_VALUE as usize..].not_any());
    }
}

impl<'a, Base: SudokuBase> Drop for CandidatesMut<'a, Base> {
    fn drop(&mut self) {
        self.debug_assert();
    }
}

#[cfg(test)]
mod tests {
    use typenum::consts::*;

    use crate::error::Result;

    use super::*;

    #[test]
    fn test_try_from_vec_u8() -> Result<()> {
        let vec_candidates = vec![1, 2, 4, 8, 9];

        let candidates = Candidates::<U3>::try_from(vec_candidates.clone())?;
        assert_eq!(candidates.to_vec_u8(), vec_candidates);

        let candidates = Candidates::<U3>::try_from(Vec::<u8>::new())?;
        assert_eq!(candidates.to_vec_u8(), Vec::<u8>::new());

        let candidates = Candidates::<U3>::try_from(vec![0]);
        assert!(candidates.is_err());

        let candidates = Candidates::<U3>::try_from(vec![10]);
        assert!(candidates.is_err());

        Ok(())
    }
}
