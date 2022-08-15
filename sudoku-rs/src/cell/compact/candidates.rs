use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use bitvec::prelude::*;
use bitvec::view::{BitView, BitViewSized};

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::error::{Error, Result};

type CandidatesBitSlice<Base> = BitSlice<<<Base as SudokuBase>::CandidatesArray as BitView>::Store>;

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

/// Constructors
impl<Base: SudokuBase> Candidates<Base> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_integral(int: Base::CandidatesIntegral) -> Self {
        let mut this = Self::new();
        this.arr.store_le(int);
        this.assert_is_valid();
        this
    }

    pub fn single(candidate: Value<Base>) -> Self {
        let mut this = Self::new();
        this.set(candidate, true);
        this
    }

    pub fn all() -> Self {
        let mut this = Self::default();

        this.arr[0..Base::MAX_VALUE as usize].fill(true);

        this.debug_assert_is_valid();

        this
    }
}

/// Mutations
impl<Base: SudokuBase> Candidates<Base> {
    pub fn toggle(&mut self, candidate: Value<Base>) {
        let imported_candidate = Self::import(candidate);

        let toggled_bit = !self.arr[imported_candidate];
        self.arr.set(imported_candidate, toggled_bit);

        self.debug_assert_is_valid();
    }

    pub fn set(&mut self, candidate: Value<Base>, value: bool) {
        let imported_candidate = Self::import(candidate);

        self.arr.set(imported_candidate, value);

        self.debug_assert_is_valid();
    }

    pub fn delete(&mut self, candidate: Value<Base>) {
        let imported_candidate = Self::import(candidate);

        self.arr.set(imported_candidate, false);

        self.debug_assert_is_valid();
    }
}

/// Getters
impl<Base: SudokuBase> Candidates<Base> {
    pub fn has(&self, candidate: Value<Base>) -> bool {
        let imported_candidate = Self::import(candidate);

        self.arr[imported_candidate]
    }

    pub fn integral(&self) -> Base::CandidatesIntegral {
        self.arr.load_le()
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

/// Internal helpers
impl<Base: SudokuBase> Candidates<Base> {
    fn import(candidate: Value<Base>) -> usize {
        (candidate.into_u8() - 1).into()
    }

    fn export(candidate: usize) -> Value<Base> {
        u8::try_from(candidate + 1).unwrap().try_into().unwrap()
    }

    fn debug_assert_is_valid(&self) {
        debug_assert!(
            self.is_valid(),
            "Unexpected bit set in {}",
            self.unused_bits()
        );
    }

    fn assert_is_valid(&self) {
        assert!(
            self.is_valid(),
            "Unexpected bit set in {}",
            self.unused_bits()
        );
    }

    fn is_valid(&self) -> bool {
        self.unused_bits().not_any()
    }

    fn unused_bits(&self) -> &CandidatesBitSlice<Base> {
        &self.arr[Base::MAX_VALUE as usize..]
    }
}

impl<Base: SudokuBase> From<Vec<Value<Base>>> for Candidates<Base> {
    fn from(candidates: Vec<Value<Base>>) -> Self {
        let mut this = Self::default();

        for candidate in candidates {
            this.arr.set(Self::import(candidate), true);
        }

        this.debug_assert_is_valid();

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

        this.debug_assert_is_valid();

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
    use std::mem::size_of;
    use typenum::consts::*;

    use crate::error::Result;

    use super::*;

    #[test]
    fn test_single() {
        assert_eq!(
            Candidates::<U2>::single(3.try_into().unwrap()).to_vec_u8(),
            vec![3]
        );
    }

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

    #[test]
    fn test_size() {
        assert_eq!(
            vec![
                size_of::<Candidates<U1>>(),
                size_of::<Candidates<U2>>(),
                size_of::<Candidates<U3>>(),
                size_of::<Candidates<U4>>(),
                size_of::<Candidates<U5>>()
            ],
            vec![1, 1, 2, 2, 4,]
        )
    }

    #[test]
    fn test_integral() {
        type Base = U5;

        let mut candidates = Candidates::<Base>::new();
        assert_eq!(candidates.integral(), 0);
        assert_eq!(candidates.to_vec_u8(), vec![]);
        candidates.set(1.try_into().unwrap(), true);
        assert_eq!(candidates.integral(), 1);
        assert_eq!(candidates.to_vec_u8(), vec![1]);
        candidates.set(2.try_into().unwrap(), true);
        assert_eq!(candidates.integral(), 3);
        assert_eq!(candidates.to_vec_u8(), vec![1, 2]);
        let mut candidates = Candidates::<Base>::new();
        candidates.set(25.try_into().unwrap(), true);
        assert_eq!(candidates.to_vec_u8(), vec![25]);
        assert_eq!(candidates.integral(), 1 << 24);
        candidates.set(10.try_into().unwrap(), true);
        assert_eq!(candidates.to_vec_u8(), vec![10, 25]);
        assert_eq!(candidates.integral(), 1 << 24 | 1 << 9);
    }

    #[test]
    fn test_with_integral() {
        type Base = U5;
        fn assert_integral_identity(i: u32) {
            assert_eq!(Candidates::<Base>::with_integral(i).integral(), i);
        }

        let powers_of_two = std::iter::successors(Some(1u32), |i| {
            let next = i << 1;
            if next >= 2u32.pow(Base::MAX_VALUE.into()) {
                None
            } else {
                Some(next)
            }
        });

        for i in powers_of_two {
            assert_integral_identity(i);
            assert_eq!(
                Candidates::<Base>::with_integral(i).to_vec_u8(),
                vec![(i.trailing_zeros() + 1).try_into().unwrap()]
            );
        }
        assert_integral_identity(0);
        assert_eq!(Candidates::<Base>::with_integral(0).to_vec_u8(), vec![]);
        let all = 0b0000_0001_1111_1111_1111_1111_1111_1111;
        assert_integral_identity(all);
        assert_eq!(
            Candidates::<Base>::with_integral(all).to_vec_u8(),
            (1..=25).collect::<Vec<_>>()
        );
    }
}
