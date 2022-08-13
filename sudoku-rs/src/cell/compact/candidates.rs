use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use bitvec::prelude::*;

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::error::{Error, Result};

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

        this.arr[0..Base::MAX_VALUE as usize].fill(true);

        this.debug_assert();

        this
    }

    pub fn toggle(&mut self, candidate: Value<Base>) {
        let imported_candidate = Self::import(candidate);

        let toggled_bit = !self.arr[imported_candidate];
        self.arr.set(imported_candidate, toggled_bit);

        self.debug_assert();
    }

    pub fn set(&mut self, candidate: Value<Base>, value: bool) {
        let imported_candidate = Self::import(candidate);

        self.arr.set(imported_candidate, value);

        self.debug_assert();
    }

    pub fn delete(&mut self, candidate: Value<Base>) {
        let imported_candidate = Self::import(candidate);

        self.arr.set(imported_candidate, false);

        self.debug_assert();
    }

    pub fn iter(&self) -> impl Iterator<Item = Value<Base>> + '_ {
        self.arr.iter_ones().map(|i| Self::export(i))
    }

    pub fn to_vec_u8(&self) -> Vec<u8> {
        self.iter().map(|value| value.into_u8()).collect()
    }

    pub fn to_vec_value(&self) -> Vec<Value<Base>> {
        self.iter().collect()
    }
}

impl<Base: SudokuBase> Candidates<Base> {
    fn import(candidate: Value<Base>) -> usize {
        (candidate.into_u8() - 1).into()
    }

    fn export(candidate: usize) -> Value<Base> {
        u8::try_from(candidate + 1).unwrap().try_into().unwrap()
    }

    fn debug_assert(&self) {
        debug_assert!(self.arr[Base::MAX_VALUE as usize..].not_any());
    }
}

impl<Base: SudokuBase> From<Vec<Value<Base>>> for Candidates<Base> {
    fn from(candidates: Vec<Value<Base>>) -> Self {
        let mut this = Self::default();

        for candidate in candidates {
            this.arr.set(Self::import(candidate), true);
        }

        this.debug_assert();

        this
    }
}

impl<Base: SudokuBase> TryFrom<Vec<u8>> for Candidates<Base> {
    type Error = Error;

    fn try_from(candidates: Vec<u8>) -> Result<Self> {
        let mut this = Self::default();

        for candidate in candidates {
            this.arr.set(Self::import(candidate.try_into()?), true);
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
