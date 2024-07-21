use std::fmt;
use std::fmt::{Binary, Display, Formatter};
use std::mem::size_of;
use std::ops::RangeBounds;

use anyhow::ensure;
use iter_combinations::CandidatesCombinationsIter;
use itertools::Itertools;
use num::traits::{CheckedShl, ConstOne, ConstZero, WrappingSub};
use num::{PrimInt, Zero};
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

pub use iter::{CandidatesAscIter, CandidatesIterator, CandidatesRandIter};

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::cell::dynamic::DynamicCandidates;
use crate::cell::{Cell, CellState};
use crate::error::{Error, Result};
use crate::position::{BlockCoordinate, Coordinate};

mod iter;
mod iter_combinations;

// FIXME: replace all usages of `&self` with `self`
//  `Candidates` is Copy and smaller than or equal to a 32-bit pointer
//  benchmark before/after

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug)]
pub struct Candidates<Base: SudokuBase> {
    /// # Safety invariants
    /// The bits at position `Base::MAX_VALUE` and greater must be zero.
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
        Self::with_integral_unchecked(Base::CandidatesIntegral::ZERO)
    }

    fn with_integral_unchecked(bits: Base::CandidatesIntegral) -> Self {
        let this = Self { bits };
        this.debug_assert();
        this
    }

    // TODO: refactor into TryFrom implementation
    pub fn with_integral(bits: Base::CandidatesIntegral) -> Self {
        let this = Self { bits };
        this.assert();
        this
    }

    // TODO: refactor arg `candidate: impl Into<Coordinate<Base>>`
    pub fn with_single(candidate: Value<Base>) -> Self {
        let mut this = Self::new();
        this.set(candidate, true);
        this
    }

    pub fn all() -> Self {
        Self::with_integral_unchecked(Self::all_candidates_mask())
    }

    pub fn with_range<C: Into<Coordinate<Base>> + Copy>(
        candidate_range: impl RangeBounds<C>,
    ) -> Self {
        use std::ops::Bound;

        let start_bound: Bound<Coordinate<Base>> = candidate_range.start_bound().map(|&c| c.into());
        let end_bound: Bound<Coordinate<Base>> = candidate_range.end_bound().map(|&c| c.into());

        let excluded_mask = match start_bound {
            // TODO: add Coordinate::{inc|dec}() -> Option<Coordinate>
            Bound::Included(start) if start == Coordinate::default() => {
                Base::CandidatesIntegral::ZERO
            }
            Bound::Included(start) => Self::all_less_than_or_equal_candidates_mask(
                // Safety: the previous match arm checks for zero. Therefore, the expression remains in-bounds and doesn't underflow.
                unsafe { Coordinate::new_unchecked(start.get() - 1) },
            ),
            Bound::Excluded(start) => Self::all_less_than_or_equal_candidates_mask(start),
            Bound::Unbounded => Base::CandidatesIntegral::ZERO,
        };

        let included_mask = match end_bound {
            Bound::Included(end) => Self::all_less_than_or_equal_candidates_mask(end),
            Bound::Excluded(end) if end == Coordinate::default() => Base::CandidatesIntegral::ZERO,
            Bound::Excluded(end) => Self::all_less_than_or_equal_candidates_mask(
                // Safety: the previous match arm checks for zero. Therefore, the expression remains in-bounds and doesn't underflow.
                unsafe { Coordinate::new_unchecked(end.get() - 1) },
            ),
            Bound::Unbounded => Self::all_candidates_mask(),
        };

        Self::with_integral_unchecked(included_mask)
            .without(Self::with_integral_unchecked(excluded_mask))
    }

    pub fn block_segmentation_mask(segment_index: BlockCoordinate<Base>) -> Self {
        let base = Base::BASE;
        let one = Base::CandidatesIntegral::ONE;

        let first_segment_mask = (one << base) - one;

        Self::with_integral_unchecked(first_segment_mask << (segment_index.get() * base))
    }

    pub fn iter_all_lexicographical() -> impl Iterator<Item = Self> + Clone {
        num::range_step_inclusive(
            Base::CandidatesIntegral::ZERO,
            Self::all_candidates_mask(),
            Base::CandidatesIntegral::ONE,
        )
        .map(Self::with_integral_unchecked)
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
    pub fn toggle(&mut self, candidate: impl Into<Coordinate<Base>>) {
        let candidate_mask = Self::import(candidate);

        self.bits ^= candidate_mask;

        self.debug_assert();
    }

    pub fn set(&mut self, candidate: impl Into<Coordinate<Base>>, enabled: bool) {
        let candidate_mask = Self::import(candidate);

        if enabled {
            self.bits |= candidate_mask;
        } else {
            self.bits &= !candidate_mask;
        }

        self.debug_assert();
    }

    pub fn set_range<C: Into<Coordinate<Base>> + Copy>(
        &mut self,
        candidate_range: impl RangeBounds<C>,
        enabled: bool,
    ) {
        let range_candidates = Self::with_range(candidate_range);
        if enabled {
            *self = self.union(range_candidates);
        } else {
            *self = self.without(range_candidates);
        }
        self.debug_assert();
    }

    pub fn insert(&mut self, candidate: impl Into<Coordinate<Base>>) {
        self.set(candidate, true);
    }

    pub fn delete(&mut self, candidate: impl Into<Coordinate<Base>>) {
        self.set(candidate, false);
    }
}

/// Getters
impl<Base: SudokuBase> Candidates<Base> {
    pub fn has(&self, candidate: impl Into<Coordinate<Base>>) -> bool {
        let candidate_mask = Self::import(candidate);

        !(self.bits & candidate_mask).is_zero()
    }

    pub fn integral(self) -> Base::CandidatesIntegral {
        self.bits
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_zero()
    }

    pub fn is_single(self) -> bool {
        self.count() == 1
    }

    pub fn is_full(&self) -> bool {
        self.bits == Self::all_candidates_mask()
    }

    fn trailing_zeros(&self) -> u8 {
        debug_assert!(!self.is_empty());
        // unwrap optimizes away
        self.bits.trailing_zeros().try_into().unwrap()
    }

    /// Logical leading zeros, e.g. only with respect to `all_candidates_mask`.
    ///
    /// Who many candidates exist after the last set candidate?
    fn leading_zeros(&self) -> u8 {
        debug_assert!(!self.is_empty());

        // unwrap optimizes away
        u8::try_from(self.bits.leading_zeros()).unwrap()
            - (const { Self::storage_bit_count() - Base::MAX_VALUE - 1 })
    }

    // Reference: https://lemire.me/blog/2018/02/21/iterating-over-set-bits-quickly/
    pub fn first<C: From<Coordinate<Base>>>(&self) -> Option<C> {
        if self.is_empty() {
            None
        } else {
            let candidate = self.trailing_zeros();

            // Safety: the largest bit position is `Base::MAX_VALUE - 1`
            // At least one bit is set, therefore `candidate` remains in-bounds.
            let coordinate = unsafe { Coordinate::new_unchecked(candidate) };

            Some(Self::export(coordinate))
        }
    }

    pub fn last<C: From<Coordinate<Base>>>(&self) -> Option<C> {
        if self.is_empty() {
            None
        } else {
            // Safety: the largest bit position is `Base::MAX_VALUE - 1`
            // At least one bit is set, therefore `candidate` remains in-bounds.
            let coordinate =
                unsafe { Coordinate::new_unchecked(Base::MAX_VALUE - self.leading_zeros()) };

            Some(Self::export(coordinate))
        }
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

        let storage_leading_zeros: u8 = self.bits.leading_zeros().try_into().unwrap();
        // Check if empty
        if storage_leading_zeros == Self::storage_bit_count() {
            return None;
        }
        let all_leading_zeros: u8 = Self::all_candidates_mask()
            .leading_zeros()
            .try_into()
            .unwrap();
        let logic_leading_zeros = storage_leading_zeros - all_leading_zeros;
        let trailing_zeros = self.trailing_zeros();
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
        (start_block_coordinate == end_coordinate).then_some(start_block_coordinate)
    }

    pub fn count(&self) -> u8 {
        self.bits.count_ones().try_into().unwrap()
    }

    pub fn iter(self) -> CandidatesAscIter<Base> {
        self.into_iter()
    }

    pub fn to_vec_u8(self) -> Vec<u8> {
        self.iter().map(|value| value.get()).collect()
    }

    pub fn to_vec_value(self) -> Vec<Value<Base>> {
        self.iter().collect()
    }

    pub fn to_single(self) -> Option<Value<Base>> {
        let mut iter = self.iter();
        let (Some(single), None) = (iter.next(), iter.next()) else {
            return None;
        };
        Some(single)
    }

    #[must_use]
    pub fn invert(self) -> Self {
        Self::with_integral_unchecked(self.bits ^ Self::all_candidates_mask())
    }

    /// Returns an iterator over all combinations of `k` candidates contained in this `Candidates`.
    pub fn combinations(self, k: Value<Base>) -> CandidatesCombinationsIter<Base> {
        CandidatesCombinationsIter::new(k, self)
    }
}

/// Internal helpers
impl<Base: SudokuBase> Candidates<Base> {
    fn all_candidates_mask() -> Base::CandidatesIntegral {
        Self::all_less_than_or_equal_candidates_mask(Coordinate::max())
    }
    fn all_less_than_or_equal_candidates_mask(
        candidate: Coordinate<Base>,
    ) -> Base::CandidatesIntegral {
        let zero = Base::CandidatesIntegral::ZERO;
        let one = Base::CandidatesIntegral::ONE;
        // Pedantic micro-optimization:
        // We only require explicit overflow handling for base4,
        // where MAX_VALUE matches the storage bit count *exactly*.
        // For other bases we can make use of the unsed higher bits and execute unchecked shifts/subtraction.
        // This results in a "huge" difference of
        // Base4: 8 x86 ASM instructions, including CMP and CMOVB
        // Else: 4 x86 ASM instructions, no comparisons or condidional move.
        if Base::MAX_VALUE == Self::storage_bit_count() {
            one.checked_shl(u32::from(candidate.get() + 1))
                .unwrap_or(zero)
                .wrapping_sub(&one)
        } else {
            one.unsigned_shl(u32::from(candidate.get() + 1)) - one
        }
    }
    // const TryInto is unstable
    #[allow(clippy::cast_possible_truncation)]
    const fn storage_bit_count() -> u8 {
        let storage_bit_count_usize = size_of::<Base::CandidatesIntegral>() * 8;
        assert!(
            storage_bit_count_usize <= u8::MAX as usize,
            "Unexpected overflow of storage_bit_count_usize"
        );
        storage_bit_count_usize as u8
    }

    /// Convert a single candidate (`Value` or `Coordinate`) into a bit mask.
    fn import(candidate: impl Into<Coordinate<Base>>) -> Base::CandidatesIntegral {
        Base::CandidatesIntegral::ONE << candidate.into().get()
    }

    fn export<C: From<Coordinate<Base>>>(candidate: Coordinate<Base>) -> C {
        candidate.into()
    }
}

/// Validation
impl<Base: SudokuBase> Candidates<Base> {
    fn mask_unused_bits(bits: Base::CandidatesIntegral) -> Base::CandidatesIntegral {
        bits & !Self::all_candidates_mask()
    }

    fn validate_integral(bits: Base::CandidatesIntegral) -> Result<()> {
        let unused_bits = Self::mask_unused_bits(bits);
        ensure!(unused_bits.is_zero(), "Unexpected bit set in {unused_bits}");
        Ok(())
    }

    fn validate(self) -> Result<()> {
        Self::validate_integral(self.bits)
    }

    fn assert(self) {
        self.validate().unwrap();
    }

    fn debug_assert(self) {
        debug_assert!({
            self.assert();
            true
        });
    }
}

impl<Base: SudokuBase, C: Into<Coordinate<Base>>> FromIterator<C> for Candidates<Base> {
    fn from_iter<T: IntoIterator<Item = C>>(candidates: T) -> Self {
        let mut this = Self::default();

        for candidate in candidates {
            this.set(candidate, true);
        }

        this.debug_assert();

        this
    }
}

impl<Base: SudokuBase> IntoIterator for Candidates<Base> {
    type Item = Value<Base>;
    type IntoIter = CandidatesAscIter<Base>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}

#[allow(clippy::into_iter_without_iter)]
impl<Base: SudokuBase> IntoIterator for &'_ Candidates<Base> {
    type Item = Value<Base>;
    type IntoIter = CandidatesAscIter<Base>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Base: SudokuBase> From<Vec<Value<Base>>> for Candidates<Base> {
    fn from(candidates: Vec<Value<Base>>) -> Self {
        candidates.into_iter().collect()
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
            this.set(Value::try_from(candidate)?, true);
        }

        this.debug_assert();

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
        if f.alternate() {
            write!(f, "{}", self.to_vec_value().into_iter().join(","))
        } else {
            write!(
                f,
                "{}",
                self.to_vec_value()
                    .into_iter()
                    .map(|value| value.to_string())
                    .join(",")
            )
        }
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
        for candidate in self {
            seq.serialize_element(&candidate)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
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
        fn test_with_range() {
            use std::fmt::Debug;
            use std::ops::{Bound, RangeBounds};

            fn assert_range_eq<Base: SudokuBase>(
                range: impl RangeBounds<Value<Base>> + Clone + Debug,
            ) {
                let candidates = Candidates::with_range(range.clone());
                assert!(candidates
                    .into_iter()
                    .all(|candidate| range.contains(&candidate)));
                let inverted_candidates = candidates.invert();
                assert!(inverted_candidates
                    .into_iter()
                    .all(|candidate| !range.contains(&candidate)));
            }

            fn all_bounds<Base: SudokuBase>() -> impl Iterator<Item = Bound<Value<Base>>> + Clone {
                Value::all()
                    .map(Bound::Included)
                    .chain(Value::all().map(Bound::Excluded))
                    .chain(std::iter::once(Bound::Unbounded))
            }

            // Sample tests
            assert_eq!(
                Candidates::<Base2>::with_range::<Value<_>>(..),
                Candidates::all()
            );
            assert_eq!(
                Candidates::<Base2>::with_range::<Value<_>>(Value::default()..),
                Candidates::all()
            );
            assert_eq!(
                Candidates::<Base2>::with_range::<Value<_>>(..=Value::max()),
                Candidates::all()
            );
            assert_eq!(
                Candidates::<Base2>::with_range::<Value<_>>(Value::default()..=Value::max()),
                Candidates::all()
            );
            assert_eq!(
                Candidates::<Base2>::with_range::<Value<_>>(..=Value::default()),
                Candidates::with_single(Value::default())
            );
            assert_eq!(
                Candidates::<Base2>::with_range::<Value<_>>(Value::default()..=Value::default()),
                Candidates::with_single(Value::default())
            );
            assert_eq!(
                Candidates::<Base2>::with_range::<Value<_>>(
                    Value::default()..Value::try_from(2).unwrap()
                ),
                Candidates::with_single(Value::default())
            );

            // Exhasitve tests for base 2 and 3
            for range in all_bounds::<Base2>().cartesian_product(all_bounds()) {
                assert_range_eq(range);
            }
            for range in all_bounds::<Base3>().cartesian_product(all_bounds()) {
                assert_range_eq(range);
            }
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
        fn test_iter_all_lexicographical() {
            itertools::assert_equal(
                Candidates::<Base2>::iter_all_lexicographical(),
                (0..16).map(Candidates::with_integral),
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
            candidates.unwrap_err();

            let candidates = Candidates::<Base3>::try_from(vec![10]);
            candidates.unwrap_err();

            Ok(())
        }
    }

    mod mutations {
        use super::*;

        #[test]
        fn test_toggle() {
            let mut candidates = Candidates::<Base2>::new();
            let value1 = Value::try_from(1).unwrap();
            let value2 = Value::try_from(2).unwrap();
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
            let value1 = Value::try_from(1).unwrap();
            let value2 = Value::try_from(2).unwrap();
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
        fn test_set_range() {
            let mut candidates = Candidates::<Base2>::new();
            let value2 = Value::try_from(2).unwrap();
            let all = ..;
            let first_half = ..=value2;
            let second_half = value2..;

            let candidates_empty = Candidates::new();
            let candidates_first_half = vec![1, 2].try_into().unwrap();
            let candidates_second_half = vec![3, 4].try_into().unwrap();
            let candidates_all = Candidates::all();

            candidates.set_range::<Value<_>>(all, false);
            assert_eq!(candidates, candidates_empty);
            candidates.set_range(first_half, true);
            assert_eq!(candidates, candidates_first_half);
            candidates.set_range(second_half.clone(), true);
            assert_eq!(candidates, candidates_all);
            candidates.set_range(first_half, false);
            assert_eq!(candidates, candidates_second_half);
            candidates.set_range(second_half, false);
            assert_eq!(candidates, candidates_empty);
        }

        #[test]
        fn test_insert() {
            let mut candidates = Candidates::<Base2>::new();
            let value1 = Value::try_from(1).unwrap();
            let value2 = Value::try_from(2).unwrap();
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
            let value1 = Value::try_from(1).unwrap();
            let value2 = Value::try_from(2).unwrap();
            candidates.delete(value2);
            assert_eq!(candidates.to_vec_u8(), vec![1, 3, 4]);
            candidates.delete(value1);
            assert_eq!(candidates.to_vec_u8(), vec![3, 4]);
            candidates.delete(value1);
            assert_eq!(candidates.to_vec_u8(), vec![3, 4]);
        }
    }

    mod getters {
        use std::{collections::BTreeSet, iter};

        use super::*;

        #[test]
        fn test_has() {
            let candidates: Candidates<Base2> = vec![1, 3].try_into().unwrap();
            let value1 = Value::try_from(1).unwrap();
            let value2 = Value::try_from(2).unwrap();
            let value3 = Value::try_from(3).unwrap();
            let value4 = Value::try_from(4).unwrap();
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
            candidates.set(Value::try_from(1).unwrap(), true);
            assert_eq!(candidates.integral(), 1);
            assert_eq!(candidates.to_vec_u8(), vec![1]);
            candidates.set(Value::try_from(2).unwrap(), true);
            assert_eq!(candidates.integral(), 3);
            assert_eq!(candidates.to_vec_u8(), vec![1, 2]);
            let mut candidates = Candidates::<Base>::new();
            candidates.set(Value::try_from(25).unwrap(), true);
            assert_eq!(candidates.to_vec_u8(), vec![25]);
            assert_eq!(candidates.integral(), 1 << 24);
            candidates.set(Value::try_from(10).unwrap(), true);
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
        fn test_is_single() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert!(!empty.is_single());
            assert!(one.is_single());
            assert!(!all.is_single());
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

        #[test]
        fn test_first() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let four: Candidates<Base2> = Candidates::with_single(4.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert!(empty.first::<Value<_>>().is_none());
            assert_eq!(one.first(), Some(Value::try_from(1).unwrap()));
            assert_eq!(four.first(), Some(Value::try_from(4).unwrap()));
            assert_eq!(all.first(), Some(Value::try_from(1).unwrap()));
        }

        #[test]
        fn test_last() {
            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let four: Candidates<Base2> = Candidates::with_single(4.try_into().unwrap());
            let all: Candidates<Base2> = Candidates::all();

            assert!(empty.last::<Value<_>>().is_none());
            assert_eq!(one.last(), Some(Value::try_from(1).unwrap()));
            assert_eq!(four.last(), Some(Value::try_from(4).unwrap()));
            assert_eq!(all.last(), Some(Value::try_from(4).unwrap()));
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
                Base::CandidatesIntegral::ZERO,
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

        #[test]
        fn test_combinations() {
            fn assert_set_equals<Base: SudokuBase>(
                a: impl IntoIterator<Item = Candidates<Base>>,
                b: impl IntoIterator<Item = Candidates<Base>>,
            ) {
                let a: BTreeSet<_> = a.into_iter().collect();
                let b: BTreeSet<_> = b.into_iter().collect();
                assert_eq!(a, b);
            }

            fn combinations_itertools<Base: SudokuBase>(
                candidates: Candidates<Base>,
                k: Value<Base>,
            ) -> impl Iterator<Item = Candidates<Base>> {
                use itertools::Itertools;

                candidates
                    .iter()
                    .combinations(k.get().into())
                    .map(|combination| combination.into())
            }

            let empty: Candidates<Base2> = Candidates::new();
            let one: Candidates<Base2> = Candidates::with_single(1.try_into().unwrap());
            let one_two_three: Candidates<Base2> = vec![1, 2, 3].try_into().unwrap();
            let all: Candidates<Base2> = Candidates::all();
            let all_base3: Candidates<Base3> = Candidates::all();

            for k in Value::all() {
                // empty candidates never produce any combinations
                itertools::assert_equal(empty.combinations(k), iter::empty());

                if k.get() == 1 {
                    // a single candidate only produces a single combination if k == 1
                    itertools::assert_equal(one.combinations(k), iter::once(one));
                } else {
                    // a single candidate never produces any combinations if k > 1
                    itertools::assert_equal(one.combinations(k), iter::empty());
                }
            }

            assert_set_equals(
                all.combinations(1.try_into().unwrap()),
                all.iter().map(Candidates::with_single),
            );

            assert_set_equals(
                one_two_three.combinations(2.try_into().unwrap()),
                vec![
                    vec![1, 2].try_into().unwrap(),
                    vec![1, 3].try_into().unwrap(),
                    vec![2, 3].try_into().unwrap(),
                ],
            );
            assert_set_equals(
                one_two_three.combinations(3.try_into().unwrap()),
                iter::once(one_two_three),
            );

            for k in Value::all() {
                assert_set_equals(empty.combinations(k), combinations_itertools(empty, k));
                assert_set_equals(one.combinations(k), combinations_itertools(one, k));
                assert_set_equals(
                    one_two_three.combinations(k),
                    combinations_itertools(one_two_three, k),
                );
                assert_set_equals(all.combinations(k), combinations_itertools(all, k));
            }
            for k in Value::all() {
                assert_set_equals(
                    all_base3.combinations(k),
                    combinations_itertools(all_base3, k),
                );
            }
        }

        #[test]
        fn test_combinations_debug() {
            dbg!(Candidates::<Base3>::all()
                .combinations(3.try_into().unwrap())
                .map(|candidates| candidates.to_vec_u8())
                .collect::<Vec<_>>());
        }
    }

    mod internal_helpers {
        use crate::test_util::for_all_bases;

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

        #[test]
        fn test_all_less_than_or_equal_candidates_mask() {
            fn assert_all_less_than_or_equal_candidates_mask<Base: SudokuBase>(
                candidate: Coordinate<Base>,
            ) {
                let value = Value::from(candidate);
                let candidates = Candidates::<Base>::with_integral(
                    Candidates::<Base>::all_less_than_or_equal_candidates_mask(candidate),
                );
                assert_eq!(
                    candidates.to_vec_u8(),
                    (1..=value.get()).collect::<Vec<_>>()
                );
            }

            for_all_bases! {
                for candidate in Coordinate::<Base>::all() {
                    assert_all_less_than_or_equal_candidates_mask(candidate);
                }
            }
        }

        #[test]
        fn test_storage_bit_count() {
            assert_eq!(Candidates::<Base2>::storage_bit_count(), 8);
            assert_eq!(Candidates::<Base3>::storage_bit_count(), 16);
            assert_eq!(Candidates::<Base4>::storage_bit_count(), 16);
            assert_eq!(Candidates::<Base5>::storage_bit_count(), 32);
        }
    }
}
