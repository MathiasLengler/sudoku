use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Binary, Display, Formatter};
use std::mem::size_of;

use num::traits::{CheckedShl, WrappingSub};
use num::{One, PrimInt, Zero};
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

pub use iter::CandidatesIter;

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::cell::dynamic::DynamicCandidates;
use crate::cell::{Cell, CellState};
use crate::error::{Error, Result};
use crate::position::{BlockCoordinate, Coordinate};

mod iter;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub struct Candidates<Base: SudokuBase> {
    /// # Safety invariants
    ///
    /// The bits at position `Base::MAX_VALUE` and greater must be zero.
    /// Validated by `Self::is_valid`.
    bits: Base::CandidatesIntegral,
}

impl<Base: SudokuBase> Default for Candidates<Base> {
    fn default() -> Self {
        Self::new()
    }
}

/// Constructors
impl<Base: SudokuBase> Candidates<Base> {
    pub fn new() -> Self {
        Self::with_integral_unchecked(Base::CandidatesIntegral::zero())
    }

    fn with_integral_unchecked(bits: Base::CandidatesIntegral) -> Self {
        let this = Self { bits };
        this.debug_assert_is_valid();
        this
    }

    // TODO: refactor into TryFrom implementation
    pub fn with_integral(bits: Base::CandidatesIntegral) -> Self {
        let this = Self { bits };
        this.assert_is_valid();
        this
    }

    pub fn with_single(candidate: Value<Base>) -> Self {
        let mut this = Self::new();
        this.set(candidate, true);
        this
    }

    pub fn all() -> Self {
        Self::with_integral_unchecked(Self::all_candidates_mask())
    }

    pub fn block_segmentation_mask(segment_index: BlockCoordinate<Base>) -> Self {
        let base = Base::BASE;
        let one = Base::CandidatesIntegral::one();

        let first_segment_mask = (one << base) - one;

        Self::with_integral_unchecked(first_segment_mask << (segment_index.get() * base))
    }
}

/// Set constructors
impl<Base: SudokuBase> Candidates<Base> {
    /// `self ∪ other`
    ///
    /// [Reference](https://en.wikipedia.org/wiki/Union_(set_theory))
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self::with_integral_unchecked(self.bits | other.bits)
    }

    /// `self ∩ other`
    ///
    /// [Reference](https://en.wikipedia.org/wiki/Intersection_(set_theory))
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        Self::with_integral_unchecked(self.bits & other.bits)
    }

    /// `self ∖ other`
    ///
    /// [Reference](https://en.wikipedia.org/wiki/Complement_(set_theory)#Relative_complement)
    #[must_use]
    pub fn without(self, other: Self) -> Self {
        Self::with_integral_unchecked(self.bits & !other.bits)
    }
}

/// Mutations
impl<Base: SudokuBase> Candidates<Base> {
    pub fn toggle(&mut self, candidate: Value<Base>) {
        let imported_candidate = Self::import(candidate);

        self.bits ^= Base::CandidatesIntegral::one() << imported_candidate;

        self.debug_assert_is_valid();
    }

    pub fn set(&mut self, candidate: Value<Base>, enabled: bool) {
        let imported_candidate = Self::import(candidate);

        if enabled {
            self.bits |= Base::CandidatesIntegral::one() << imported_candidate;
        } else {
            self.bits &= !(Base::CandidatesIntegral::one() << imported_candidate);
        }

        self.debug_assert_is_valid();
    }

    pub fn insert(&mut self, candidate: Value<Base>) {
        self.set(candidate, true);
    }

    pub fn delete(&mut self, candidate: Value<Base>) {
        self.set(candidate, false);
    }
}

/// Getters
impl<Base: SudokuBase> Candidates<Base> {
    pub fn has(&self, candidate: Value<Base>) -> bool {
        let imported_candidate = Self::import(candidate);

        (self.bits & Base::CandidatesIntegral::one() << imported_candidate)
            != Base::CandidatesIntegral::zero()
    }

    pub fn integral(&self) -> Base::CandidatesIntegral {
        self.bits
    }

    pub fn is_empty(&self) -> bool {
        self.bits == Base::CandidatesIntegral::zero()
    }

    pub fn is_full(&self) -> bool {
        self.bits == Self::all_candidates_mask()
    }

    /// Determine if the candidates are block segmented or not.
    ///
    /// Candidates are block segmented, if:
    /// - The candidates contain `2..=Base::BASE` candidates
    /// - All candidates are contained in a single block segment,
    ///   e.g. have a bit position in the range of `(n..n+Base::BASE)`, where n is some non-negative integer.
    ///
    /// If the candidates are block segmented, the block segment index is returned, otherwise `None`.
    pub fn block_segmentation(self) -> Option<BlockCoordinate<Base>> {
        let base = Base::BASE;

        let size_bits: u8 = (size_of::<Base::CandidatesIntegral>() * 8)
            .try_into()
            .unwrap();
        let storage_leading_zeros: u8 = self.bits.leading_zeros().try_into().unwrap();
        // Check if empty
        if storage_leading_zeros == size_bits {
            return None;
        }
        let all_leading_zeros: u8 = Self::all_candidates_mask()
            .leading_zeros()
            .try_into()
            .unwrap();
        let logic_leading_zeros = storage_leading_zeros - all_leading_zeros;
        let trailing_zeros: u8 = self.bits.trailing_zeros().try_into().unwrap();
        let outer_zeros_count = logic_leading_zeros + trailing_zeros;

        let segment_width = Base::SIDE_LENGTH - outer_zeros_count;
        if !(2..=base).contains(&segment_width) {
            return None;
        }
        // Check for misaligned segment
        let start = trailing_zeros;
        // Safety: a non-empty Candidates always has less than Base::SIDE_LENGTH leading zeros.
        let start_coordinate = unsafe { Coordinate::<Base>::new_unchecked(start) };
        let start_block_coordinate = BlockCoordinate::round_down(start_coordinate);

        let end = Base::SIDE_LENGTH - (logic_leading_zeros + 1);
        // Safety: `end` is always less than `Base::SIDE_LENGTH`,
        // because `logic_leading_zeros` is less than `Base::SIDE_LENGTH` for non-empty Candidates.
        let end_coordinate = unsafe { Coordinate::<Base>::new_unchecked(end) };
        let end_coordinate = BlockCoordinate::round_down(end_coordinate);
        if start_block_coordinate == end_coordinate {
            Some(start_block_coordinate)
        } else {
            None
        }
    }

    pub fn count(&self) -> u8 {
        self.bits.count_ones().try_into().unwrap()
    }

    pub fn iter(&self) -> CandidatesIter<Base> {
        (*self).into()
    }

    pub fn to_vec_u8(&self) -> Vec<u8> {
        self.iter().map(|value| value.get()).collect()
    }

    pub fn to_vec_value(&self) -> Vec<Value<Base>> {
        self.iter().collect()
    }

    pub fn to_single(self) -> Option<Value<Base>> {
        let mut iter = self.iter();
        let (Some(single), None) = (iter.next(), iter.next()) else { return None; };
        Some(single)
    }

    #[must_use]
    pub fn invert(self) -> Self {
        Self::with_integral_unchecked(self.bits ^ Self::all_candidates_mask())
    }
}

/// Internal helpers
impl<Base: SudokuBase> Candidates<Base> {
    fn all_candidates_mask() -> Base::CandidatesIntegral {
        let zero = Base::CandidatesIntegral::zero();
        let one = Base::CandidatesIntegral::one();
        one.checked_shl(u32::from(Base::MAX_VALUE))
            .unwrap_or(zero)
            .wrapping_sub(&one)
    }

    fn import(candidate: Value<Base>) -> u8 {
        candidate.get() - 1
    }

    fn export(candidate: Coordinate<Base>) -> Value<Base> {
        candidate.into()
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
        self.unused_bits() == Base::CandidatesIntegral::zero()
    }

    fn unused_bits(&self) -> Base::CandidatesIntegral {
        self.bits & !Self::all_candidates_mask()
    }
}

impl<Base: SudokuBase> FromIterator<Value<Base>> for Candidates<Base> {
    fn from_iter<T: IntoIterator<Item = Value<Base>>>(candidates: T) -> Self {
        let mut this = Self::default();

        for candidate in candidates {
            this.set(candidate, true);
        }

        this.debug_assert_is_valid();

        this
    }
}

impl<Base: SudokuBase> IntoIterator for Candidates<Base> {
    type Item = Value<Base>;
    type IntoIter = CandidatesIter<Base>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

impl<Base: SudokuBase> From<Vec<Value<Base>>> for Candidates<Base> {
    fn from(candidates: Vec<Value<Base>>) -> Self {
        let mut this = Self::default();

        for candidate in candidates {
            this.set(candidate, true);
        }

        this.debug_assert_is_valid();

        this
    }
}

impl<Base: SudokuBase> From<Cell<Base>> for Candidates<Base> {
    fn from(cell: Cell<Base>) -> Self {
        match *cell.state() {
            CellState::Value(value) | CellState::FixedValue(value) => Self::with_single(value),
            CellState::Candidates(candidates) => candidates,
        }
    }
}

impl<Base: SudokuBase> TryFrom<Vec<u8>> for Candidates<Base> {
    type Error = Error;

    fn try_from(candidates: Vec<u8>) -> Result<Self> {
        let mut this = Self::default();

        for candidate in candidates {
            this.set(candidate.try_into()?, true);
        }

        this.debug_assert_is_valid();

        Ok(this)
    }
}

impl<Base: SudokuBase> TryFrom<DynamicCandidates> for Candidates<Base> {
    type Error = Error;

    fn try_from(dynamic_candidates: DynamicCandidates) -> Result<Self> {
        dynamic_candidates.0.try_into()
    }
}

impl<Base: SudokuBase> Display for Candidates<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_vec_u8())
    }
}

impl<Base: SudokuBase> Binary for Candidates<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Binary::fmt(&self.bits, f)
    }
}

impl<Base: SudokuBase> Serialize for Candidates<Base> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(usize::from(self.count())))?;
        for candidate in self.iter() {
            seq.serialize_element(&candidate)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::base::consts::*;
    use crate::error::Result;

    use super::*;

    #[test]
    fn test_size_of() {
        assert_eq!(
            vec![
                size_of::<Candidates<Base2>>(),
                size_of::<Candidates<Base3>>(),
                size_of::<Candidates<Base4>>(),
                size_of::<Candidates<Base5>>()
            ],
            vec![1, 2, 2, 4,]
        );
    }

    mod constructors {
        use super::*;

        #[test]
        fn test_new() {
            assert_eq!(Candidates::<Base2>::new().to_vec_u8(), Vec::<u8>::new());
            assert_eq!(Candidates::<Base3>::new().to_vec_u8(), Vec::<u8>::new());
            assert_eq!(Candidates::<Base4>::new().to_vec_u8(), Vec::<u8>::new());
            assert_eq!(Candidates::<Base5>::new().to_vec_u8(), Vec::<u8>::new());
        }

        #[test]
        fn test_single() {
            assert_eq!(
                Candidates::<Base2>::with_single(3.try_into().unwrap()).to_vec_u8(),
                vec![3]
            );
        }

        #[test]
        fn test_all() {
            assert_eq!(
                Candidates::<Base2>::all().to_vec_u8(),
                (1..=4).collect::<Vec<u8>>()
            );
            assert_eq!(
                Candidates::<Base3>::all().to_vec_u8(),
                (1..=9).collect::<Vec<u8>>()
            );
            assert_eq!(
                Candidates::<Base4>::all().to_vec_u8(),
                (1..=16).collect::<Vec<u8>>()
            );
            assert_eq!(
                Candidates::<Base5>::all().to_vec_u8(),
                (1..=25).collect::<Vec<u8>>()
            );
        }

        #[test]
        fn test_with_integral() {
            type Base = Base5;
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
                    vec![u8::try_from(i.trailing_zeros() + 1).unwrap()]
                );
            }
            assert_integral_identity(0);
            assert_eq!(
                Candidates::<Base>::with_integral(0).to_vec_u8(),
                Vec::<u8>::new()
            );
            let all = 0b0000_0001_1111_1111_1111_1111_1111_1111;
            assert_integral_identity(all);
            assert_eq!(
                Candidates::<Base>::with_integral(all).to_vec_u8(),
                (1..=25).collect::<Vec<_>>()
            );
        }

        #[test]
        fn test_set_constructors() {
            let a: Candidates<Base2> = vec![1, 2].try_into().unwrap();
            let b: Candidates<Base2> = vec![2, 3].try_into().unwrap();

            assert_eq!(a.union(b).to_vec_u8(), vec![1, 2, 3]);
            assert_eq!(b.union(a).to_vec_u8(), vec![1, 2, 3]);

            assert_eq!(a.intersection(b).to_vec_u8(), vec![2]);
            assert_eq!(b.intersection(a).to_vec_u8(), vec![2]);

            assert_eq!(a.without(b).to_vec_u8(), vec![1]);
            assert_eq!(b.without(a).to_vec_u8(), vec![3]);
        }

        #[test]
        fn test_try_from_vec_u8() -> Result<()> {
            let vec_candidates = vec![1, 2, 4, 8, 9];

            let candidates = Candidates::<Base3>::try_from(vec_candidates.clone())?;
            assert_eq!(candidates.to_vec_u8(), vec_candidates);

            let candidates = Candidates::<Base3>::try_from(Vec::<u8>::new())?;
            assert_eq!(candidates.to_vec_u8(), Vec::<u8>::new());

            let candidates = Candidates::<Base3>::try_from(vec![0]);
            assert!(candidates.is_err());

            let candidates = Candidates::<Base3>::try_from(vec![10]);
            assert!(candidates.is_err());

            Ok(())
        }
    }

    mod mutations {
        use super::*;

        #[test]
        fn test_toggle() {
            let mut candidates = Candidates::<Base2>::new();
            let value1 = 1.try_into().unwrap();
            let value2 = 2.try_into().unwrap();
            candidates.toggle(value1);
            assert_eq!(candidates.to_vec_u8(), vec![1]);
            candidates.toggle(value2);
            assert_eq!(candidates.to_vec_u8(), vec![1, 2]);
            candidates.toggle(value1);
            assert_eq!(candidates.to_vec_u8(), vec![2]);
            candidates.toggle(value2);
            assert_eq!(candidates.to_vec_u8(), Vec::<u8>::new());
        }

        #[test]
        fn test_set() {
            let mut candidates = Candidates::<Base2>::new();
            let value1 = 1.try_into().unwrap();
            let value2 = 2.try_into().unwrap();
            candidates.set(value1, false);
            assert_eq!(candidates.to_vec_u8(), Vec::<u8>::new());
            candidates.set(value1, true);
            assert_eq!(candidates.to_vec_u8(), vec![1]);
            candidates.set(value1, true);
            assert_eq!(candidates.to_vec_u8(), vec![1]);
            candidates.set(value2, true);
            assert_eq!(candidates.to_vec_u8(), vec![1, 2]);
            candidates.set(value1, false);
            assert_eq!(candidates.to_vec_u8(), vec![2]);
            candidates.set(value2, false);
            assert_eq!(candidates.to_vec_u8(), Vec::<u8>::new());
        }

        #[test]
        fn test_insert() {
            let mut candidates = Candidates::<Base2>::new();
            let value1 = 1.try_into().unwrap();
            let value2 = 2.try_into().unwrap();
            candidates.insert(value1);
            assert_eq!(candidates.to_vec_u8(), vec![1]);
            candidates.insert(value2);
            assert_eq!(candidates.to_vec_u8(), vec![1, 2]);
            candidates.insert(value1);
            assert_eq!(candidates.to_vec_u8(), vec![1, 2]);
        }

        #[test]
        fn test_delete() {
            let mut candidates = Candidates::<Base2>::all();
            let value1 = 1.try_into().unwrap();
            let value2 = 2.try_into().unwrap();
            candidates.delete(value2);
            assert_eq!(candidates.to_vec_u8(), vec![1, 3, 4]);
            candidates.delete(value1);
            assert_eq!(candidates.to_vec_u8(), vec![3, 4]);
            candidates.delete(value1);
            assert_eq!(candidates.to_vec_u8(), vec![3, 4]);
        }
    }

    mod getters {
        use std::collections::BTreeSet;

        use super::*;

        #[test]
        fn test_has() {
            let candidates: Candidates<Base2> = vec![1, 3].try_into().unwrap();
            let value1 = 1.try_into().unwrap();
            let value2 = 2.try_into().unwrap();
            let value3 = 3.try_into().unwrap();
            let value4 = 4.try_into().unwrap();
            assert!(candidates.has(value1));
            assert!(!candidates.has(value2));
            assert!(candidates.has(value3));
            assert!(!candidates.has(value4));
        }

        #[test]
        fn test_integral() {
            type Base = Base5;

            let mut candidates = Candidates::<Base>::new();
            assert_eq!(candidates.integral(), 0);
            assert_eq!(candidates.to_vec_u8(), Vec::<u8>::new());
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
        fn test_is_empty() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert!(empty.is_empty());
            assert!(!one.is_empty());
            assert!(!all.is_empty());
        }

        #[test]
        fn test_is_full() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert!(!empty.is_full());
            assert!(!one.is_full());
            assert!(all.is_full());
        }

        fn assert_block_segmentation<Base: SudokuBase>(
            segmented_candidates: Vec<(Base::CandidatesIntegral, u8)>,
        ) {
            for (segmented_candidates_integral, segment_index) in
                segmented_candidates.iter().copied()
            {
                assert_eq!(
                    Candidates::<Base>::with_integral(segmented_candidates_integral)
                        .block_segmentation(),
                    Some(BlockCoordinate::new(segment_index).unwrap())
                );
            }

            let segmented_integrals: BTreeSet<_> = segmented_candidates
                .into_iter()
                .map(|(integral, _)| integral)
                .collect();

            for non_segmented_integral in num::range(
                Base::CandidatesIntegral::zero(),
                Candidates::<Base>::all_candidates_mask(),
            )
            .filter(|integral| !segmented_integrals.contains(integral))
            {
                assert_eq!(
                    Candidates::<Base>::with_integral(non_segmented_integral).block_segmentation(),
                    None,
                    "Non segmented integral: {non_segmented_integral:b}"
                );
            }
        }

        #[test]
        fn test_block_segmentation_base_2() {
            let segmented_candidates = vec![(0b0011, 0), (0b1100, 1)];

            assert_block_segmentation::<Base2>(segmented_candidates);
        }

        #[test]
        fn test_block_segmentation_base_3() {
            let segmented_candidates = vec![
                (0b000_000_011, 0),
                (0b000_000_101, 0),
                (0b000_000_110, 0),
                (0b000_000_111, 0),
                (0b000_011_000, 1),
                (0b000_101_000, 1),
                (0b000_110_000, 1),
                (0b000_111_000, 1),
                (0b011_000_000, 2),
                (0b101_000_000, 2),
                (0b110_000_000, 2),
                (0b111_000_000, 2),
            ];

            assert_block_segmentation::<Base3>(segmented_candidates);
        }

        #[test]
        fn test_count() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert_eq!(empty.count(), 0);
            assert_eq!(one.count(), 1);
            assert_eq!(all.count(), 4);
        }

        #[test]
        fn test_iter() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert!(empty.iter().next().is_none());
            assert_eq!(one.iter().collect::<Vec<_>>(), vec![1.try_into().unwrap()]);
            assert_eq!(
                all.iter().collect::<Vec<_>>(),
                (1..=4)
                    .map(|i| Value::<Base2>::try_from(i).unwrap())
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn test_to_vec_u8() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert_eq!(empty.to_vec_u8(), Vec::<u8>::new());
            assert_eq!(one.to_vec_u8(), vec![1]);
            assert_eq!(all.to_vec_u8(), vec![1, 2, 3, 4]);
        }
        #[test]
        fn test_to_vec_value() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert_eq!(empty.to_vec_value(), vec![]);
            assert_eq!(one.to_vec_value(), vec![1.try_into().unwrap()]);
            assert_eq!(
                all.to_vec_value(),
                (1..=4)
                    .map(|i| Value::<Base2>::try_from(i).unwrap())
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn test_to_single() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert_eq!(empty.to_single(), None);
            assert_eq!(one.to_single(), Some(1.try_into().unwrap()));
            assert_eq!(all.to_single(), None);
        }

        #[test]
        fn test_invert() {
            let empty: Candidates<Base2> = Candidates::new();
            let all: Candidates<Base2> = Candidates::all();
            assert_eq!(empty.invert(), all);
            assert_eq!(all.invert(), empty);

            let candidates_12 = Candidates::<Base2>::with_integral(0b0011);
            let candidates_34 = Candidates::<Base2>::with_integral(0b1100);
            assert_eq!(candidates_12.invert(), candidates_34);
            assert_eq!(candidates_34.invert(), candidates_12);
        }
    }

    mod internal_helpers {
        use super::*;

        #[test]
        fn test_all_candidates_mask() {
            let all_candidates_mask = Candidates::<Base2>::all_candidates_mask();
            assert_eq!(all_candidates_mask, 0b0000_1111);
            let all_candidates_mask = Candidates::<Base3>::all_candidates_mask();
            assert_eq!(all_candidates_mask, 0b0000_0001_1111_1111);
            let all_candidates_mask = Candidates::<Base4>::all_candidates_mask();
            assert_eq!(all_candidates_mask, 0b1111_1111_1111_1111);
            let all_candidates_mask = Candidates::<Base5>::all_candidates_mask();
            assert_eq!(
                all_candidates_mask,
                0b0000_0001_1111_1111_1111_1111_1111_1111
            );
        }
    }
}
