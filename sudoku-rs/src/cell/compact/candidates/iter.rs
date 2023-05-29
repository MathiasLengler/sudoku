use num::{One, PrimInt, Zero};

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;

#[derive(Debug, Clone)]
struct IterOnes<Base: SudokuBase> {
    /// # Safety invariants
    ///
    /// The bits at position `Base::MAX_VALUE` and greater must be zero.
    bits: Base::CandidatesIntegral,
}

impl<Base: SudokuBase> From<Candidates<Base>> for IterOnes<Base> {
    fn from(candidates: Candidates<Base>) -> Self {
        Self {
            bits: candidates.bits,
        }
    }
}

impl<Base: SudokuBase> IterOnes<Base> {
    /// # Safety invariant
    ///
    /// Returned `u8` is in the range `0..Base::MAX_VALUE`.
    fn peek(&self) -> Option<u8> {
        if self.bits.is_zero() {
            None
        } else {
            let trailing_zeros = self.bits.trailing_zeros();
            // unwrap optimizes away
            let candidate = u8::try_from(trailing_zeros).unwrap();
            debug_assert!(candidate < Base::MAX_VALUE);
            Some(candidate)
        }
    }
}

// TODO: benchmark
impl<Base: SudokuBase> Iterator for IterOnes<Base> {
    type Item = u8;

    /// # Safety invariant
    ///
    /// Returned `u8` is in the range `0..Base::MAX_VALUE`.
    fn next(&mut self) -> Option<Self::Item> {
        let trailing_zeros = self.peek();
        if let Some(trailing_zeros) = trailing_zeros {
            self.bits ^= Base::CandidatesIntegral::one() << trailing_zeros;
        }
        trailing_zeros
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Base: SudokuBase> ExactSizeIterator for IterOnes<Base> {
    fn len(&self) -> usize {
        usize::try_from(self.bits.count_ones()).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct CandidatesIter<Base: SudokuBase> {
    iter: IterOnes<Base>,
}

impl<Base: SudokuBase> CandidatesIter<Base> {
    pub fn peek(&self) -> Option<Value<Base>> {
        self.iter
            .peek()
            // Safety: range of `candidate` guaranteed by `IterOnes::peek`
            .map(|candidate| unsafe { Candidates::export(candidate) })
    }
}

impl<Base: SudokuBase> Iterator for CandidatesIter<Base> {
    type Item = Value<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            // Safety: range of `candidate` guaranteed by `IterOnes::next`
            .map(|candidate| unsafe { Candidates::export(candidate) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<Base: SudokuBase> ExactSizeIterator for CandidatesIter<Base> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<Base: SudokuBase> From<Candidates<Base>> for CandidatesIter<Base> {
    fn from(candidates: Candidates<Base>) -> Self {
        Self {
            iter: candidates.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;

    use super::*;

    #[test]
    fn test_iter_ones() {
        let mut iter_ones = IterOnes::<Base2> { bits: 0b1010 };
        assert_eq!(iter_ones.clone().collect::<Vec<_>>(), vec![1, 3]);

        assert_eq!(iter_ones.peek(), Some(1));
        assert_eq!(iter_ones.peek(), Some(1));
        assert_eq!(iter_ones.next(), Some(1));

        assert_eq!(iter_ones.peek(), Some(3));
        assert_eq!(iter_ones.peek(), Some(3));
        assert_eq!(iter_ones.next(), Some(3));

        assert_eq!(iter_ones.peek(), None);
        assert_eq!(iter_ones.peek(), None);
        assert_eq!(iter_ones.next(), None);

        assert_eq!(iter_ones.peek(), None);
        assert_eq!(iter_ones.peek(), None);
        assert_eq!(iter_ones.next(), None);
    }
}
