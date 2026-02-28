use crate::base::SudokuBase;
use crate::cell::Value;
use crate::cell::compact::candidates::Candidates;
use num::traits::{
    CheckedShl, ConstOne, ConstZero, PrimInt, WrappingAdd, WrappingNeg, WrappingShr, WrappingSub,
    Zero,
};

// Reference: https://graphics.stanford.edu/%7Eseander/bithacks.html#NextBitPermutation
fn next_permutation<
    T: PrimInt + WrappingAdd + WrappingSub + WrappingNeg + WrappingShr + ConstOne,
>(
    v: T,
) -> Option<T> {
    let one = T::ONE;
    let t = v | (v.wrapping_sub(&one));

    Some(
        (t.checked_add(&one)?)
            | (!t & ((!t).wrapping_neg()))
                .wrapping_sub(&one)
                .wrapping_shr(v.trailing_zeros() + 1),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FirstCandidatesCombinationsIter<Base: SudokuBase> {
    current: Candidates<Base>,
    // `None` if iterator is finished.
    n: Option<Value<Base>>,
}

impl<Base: SudokuBase> FirstCandidatesCombinationsIter<Base> {
    /// Create a new iterator over all combinations of `k` candidates for the first `n` candidates.
    fn new(k: Value<Base>, n: Value<Base>) -> Self {
        if k > n {
            Self::new_empty()
        } else {
            Self {
                current: Candidates::with_range(..=k),
                n: Some(n),
            }
        }
    }

    fn new_empty() -> Self {
        Self {
            current: Candidates::new(),
            n: None,
        }
    }
}

// TODO: implement ExactSizeIterator (n choose k)
impl<Base: SudokuBase> Iterator for FirstCandidatesCombinationsIter<Base> {
    type Item = Candidates<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.n?;
        let current = self.current;

        let Some(next) = next_permutation(current.integral()) else {
            self.n = None;
            return Some(current);
        };

        if const { Base::BASE == 4 } && n == Value::max() {
            // Safety: this edge case is only possible for Base4 with all candidates selected.
            // Therefore, `next` is always valid. Also verified by testing.
            let next = unsafe { Candidates::with_integral_unchecked(next) };
            self.current = next;
            Some(current)
        } else {
            let mask = Base::CandidatesIntegral::ONE
                .checked_shl(n.get().into())
                .unwrap_or(Base::CandidatesIntegral::ZERO);

            if (next & mask).is_zero() {
                // Safety: masked bits are zero, so `next` is valid.
                let next = unsafe { Candidates::with_integral_unchecked(next) };
                self.current = next;
                Some(current)
            } else {
                self.n = None;
                Some(current)
            }
        }
    }
}

/// An iterator over all combinations of `k` candidates contained in a given `Candidates`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidatesCombinationsIter<Base: SudokuBase> {
    candidates: Candidates<Base>,
    iter_first: FirstCandidatesCombinationsIter<Base>,
}

impl<Base: SudokuBase> CandidatesCombinationsIter<Base> {
    pub(super) fn new(candidates: Candidates<Base>, k: Value<Base>) -> Self {
        let Some(n) = Value::new(candidates.count()).unwrap() else {
            return Self {
                candidates,
                iter_first: FirstCandidatesCombinationsIter::new_empty(),
            };
        };

        Self {
            candidates,
            iter_first: FirstCandidatesCombinationsIter::new(k, n),
        }
    }
}

impl<Base: SudokuBase> Iterator for CandidatesCombinationsIter<Base> {
    type Item = Candidates<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_first = self.iter_first.next()?;

        Some(
            (1u8..)
                .zip(self.candidates.iter())
                .filter(|&(i, _candidate)| {
                    // Safety: `Candidates` contains a maximum of `Base::MAX_VALUE` candidates.
                    // `i` starts from 1, therefore `1..=(Base::MAX_VALUE)` holds.
                    let value = unsafe { Value::new_unchecked(i) };
                    next_first.has(value)
                })
                .map(|(_i, candidate)| candidate)
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod next_permutation {
        use super::*;

        #[test]
        fn test_u8() {
            #[rustfmt::skip]
            let expected = vec![
                0b0000_0000,
                0b0000_0010,
                0b0000_0100,
                0b0000_0101,
                0b0000_1000,
                0b0000_0110,
                0b0000_1001,
                0b0000_1011,
                0b0001_0000,
                0b0000_1010,
                0b0000_1100,
                0b0000_1101,
                0b0001_0001,
                0b0000_1110,
                0b0001_0011,
                0b0001_0111,
                0b0010_0000,
                0b0001_0010,
                0b0001_0100,
                0b0001_0101,
                0b0001_1000,
                0b0001_0110,
                0b0001_1001,
                0b0001_1011,
                0b0010_0001,
                0b0001_1010,
                0b0001_1100,
                0b0001_1101,
                0b0010_0011,
                0b0001_1110,
                0b0010_0111,
                0b0010_1111,
                0b0100_0000,
                0b0010_0010,
                0b0010_0100,
                0b0010_0101,
                0b0010_1000,
                0b0010_0110,
                0b0010_1001,
                0b0010_1011,
                0b0011_0000,
                0b0010_1010,
                0b0010_1100,
                0b0010_1101,
                0b0011_0001,
                0b0010_1110,
                0b0011_0011,
                0b0011_0111,
                0b0100_0001,
                0b0011_0010,
                0b0011_0100,
                0b0011_0101,
                0b0011_1000,
                0b0011_0110,
                0b0011_1001,
                0b0011_1011,
                0b0100_0011,
                0b0011_1010,
                0b0011_1100,
                0b0011_1101,
                0b0100_0111,
                0b0011_1110,
                0b0100_1111,
                0b0101_1111,
                0b1000_0000,
                0b0100_0010,
                0b0100_0100,
                0b0100_0101,
                0b0100_1000,
                0b0100_0110,
                0b0100_1001,
                0b0100_1011,
                0b0101_0000,
                0b0100_1010,
                0b0100_1100,
                0b0100_1101,
                0b0101_0001,
                0b0100_1110,
                0b0101_0011,
                0b0101_0111,
                0b0110_0000,
                0b0101_0010,
                0b0101_0100,
                0b0101_0101,
                0b0101_1000,
                0b0101_0110,
                0b0101_1001,
                0b0101_1011,
                0b0110_0001,
                0b0101_1010,
                0b0101_1100,
                0b0101_1101,
                0b0110_0011,
                0b0101_1110,
                0b0110_0111,
                0b0110_1111,
                0b1000_0001,
                0b0110_0010,
                0b0110_0100,
                0b0110_0101,
                0b0110_1000,
                0b0110_0110,
                0b0110_1001,
                0b0110_1011,
                0b0111_0000,
                0b0110_1010,
                0b0110_1100,
                0b0110_1101,
                0b0111_0001,
                0b0110_1110,
                0b0111_0011,
                0b0111_0111,
                0b1000_0011,
                0b0111_0010,
                0b0111_0100,
                0b0111_0101,
                0b0111_1000,
                0b0111_0110,
                0b0111_1001,
                0b0111_1011,
                0b1000_0111,
                0b0111_1010,
                0b0111_1100,
                0b0111_1101,
                0b1000_1111,
                0b0111_1110,
                0b1001_1111,
                0b1011_1111,
                0b0000_0000,
                0b1000_0010,
                0b1000_0100,
                0b1000_0101,
                0b1000_1000,
                0b1000_0110,
                0b1000_1001,
                0b1000_1011,
                0b1001_0000,
                0b1000_1010,
                0b1000_1100,
                0b1000_1101,
                0b1001_0001,
                0b1000_1110,
                0b1001_0011,
                0b1001_0111,
                0b1010_0000,
                0b1001_0010,
                0b1001_0100,
                0b1001_0101,
                0b1001_1000,
                0b1001_0110,
                0b1001_1001,
                0b1001_1011,
                0b1010_0001,
                0b1001_1010,
                0b1001_1100,
                0b1001_1101,
                0b1010_0011,
                0b1001_1110,
                0b1010_0111,
                0b1010_1111,
                0b1100_0000,
                0b1010_0010,
                0b1010_0100,
                0b1010_0101,
                0b1010_1000,
                0b1010_0110,
                0b1010_1001,
                0b1010_1011,
                0b1011_0000,
                0b1010_1010,
                0b1010_1100,
                0b1010_1101,
                0b1011_0001,
                0b1010_1110,
                0b1011_0011,
                0b1011_0111,
                0b1100_0001,
                0b1011_0010,
                0b1011_0100,
                0b1011_0101,
                0b1011_1000,
                0b1011_0110,
                0b1011_1001,
                0b1011_1011,
                0b1100_0011,
                0b1011_1010,
                0b1011_1100,
                0b1011_1101,
                0b1100_0111,
                0b1011_1110,
                0b1100_1111,
                0b1101_1111,
                0b0000_0000,
                0b1100_0010,
                0b1100_0100,
                0b1100_0101,
                0b1100_1000,
                0b1100_0110,
                0b1100_1001,
                0b1100_1011,
                0b1101_0000,
                0b1100_1010,
                0b1100_1100,
                0b1100_1101,
                0b1101_0001,
                0b1100_1110,
                0b1101_0011,
                0b1101_0111,
                0b1110_0000,
                0b1101_0010,
                0b1101_0100,
                0b1101_0101,
                0b1101_1000,
                0b1101_0110,
                0b1101_1001,
                0b1101_1011,
                0b1110_0001,
                0b1101_1010,
                0b1101_1100,
                0b1101_1101,
                0b1110_0011,
                0b1101_1110,
                0b1110_0111,
                0b1110_1111,
                0b0000_0000,
                0b1110_0010,
                0b1110_0100,
                0b1110_0101,
                0b1110_1000,
                0b1110_0110,
                0b1110_1001,
                0b1110_1011,
                0b1111_0000,
                0b1110_1010,
                0b1110_1100,
                0b1110_1101,
                0b1111_0001,
                0b1110_1110,
                0b1111_0011,
                0b1111_0111,
                0b0000_0000,
                0b1111_0010,
                0b1111_0100,
                0b1111_0101,
                0b1111_1000,
                0b1111_0110,
                0b1111_1001,
                0b1111_1011,
                0b0000_0000,
                0b1111_1010,
                0b1111_1100,
                0b1111_1101,
                0b0000_0000,
                0b1111_1110,
                0b0000_0000,
                0b0000_0000,
            ];

            itertools::assert_equal(
                (0..=u8::MAX).map(next_permutation),
                expected
                    .into_iter()
                    .map(|i| if i == 0 { None } else { Some(i) }),
            );
        }
    }

    mod first_candidates_combinations_iter {
        use super::*;

        use crate::base::consts::*;
        use std::iter::{empty, once};

        mod base2 {
            use super::*;

            #[test]
            fn test_1_3() {
                let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                    1.try_into().unwrap(),
                    3.try_into().unwrap(),
                );
                itertools::assert_equal(
                    iter,
                    vec![
                        vec![1].try_into().unwrap(),
                        vec![2].try_into().unwrap(),
                        vec![3].try_into().unwrap(),
                    ],
                );
            }

            #[test]
            fn test_2_3() {
                let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                    2.try_into().unwrap(),
                    3.try_into().unwrap(),
                );
                itertools::assert_equal(
                    iter,
                    vec![
                        vec![1, 2].try_into().unwrap(),
                        vec![1, 3].try_into().unwrap(),
                        vec![2, 3].try_into().unwrap(),
                    ],
                );
            }

            #[test]
            fn test_3_3() {
                let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                    3.try_into().unwrap(),
                    3.try_into().unwrap(),
                );
                itertools::assert_equal(iter, once(vec![1, 2, 3].try_into().unwrap()));
            }

            #[test]
            fn test_4_3() {
                let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                    4.try_into().unwrap(),
                    3.try_into().unwrap(),
                );
                itertools::assert_equal(iter, empty());
            }

            #[test]
            fn test_2_4() {
                let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                    2.try_into().unwrap(),
                    4.try_into().unwrap(),
                );
                itertools::assert_equal(
                    iter,
                    vec![
                        vec![1, 2].try_into().unwrap(),
                        vec![1, 3].try_into().unwrap(),
                        vec![2, 3].try_into().unwrap(),
                        vec![1, 4].try_into().unwrap(),
                        vec![2, 4].try_into().unwrap(),
                        vec![3, 4].try_into().unwrap(),
                    ],
                );
            }

            #[test]
            fn test_3_4() {
                let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                    3.try_into().unwrap(),
                    4.try_into().unwrap(),
                );
                itertools::assert_equal(
                    iter,
                    vec![
                        vec![1, 2, 3].try_into().unwrap(),
                        vec![1, 2, 4].try_into().unwrap(),
                        vec![1, 3, 4].try_into().unwrap(),
                        vec![2, 3, 4].try_into().unwrap(),
                    ],
                );
            }
        }

        mod base3 {
            use super::*;

            #[test]
            fn test_3_5() {
                let iter = FirstCandidatesCombinationsIter::<Base3>::new(
                    3.try_into().unwrap(),
                    5.try_into().unwrap(),
                );
                itertools::assert_equal(
                    iter,
                    vec![
                        vec![1, 2, 3].try_into().unwrap(),
                        vec![1, 2, 4].try_into().unwrap(),
                        vec![1, 3, 4].try_into().unwrap(),
                        vec![2, 3, 4].try_into().unwrap(),
                        vec![1, 2, 5].try_into().unwrap(),
                        vec![1, 3, 5].try_into().unwrap(),
                        vec![2, 3, 5].try_into().unwrap(),
                        vec![1, 4, 5].try_into().unwrap(),
                        vec![2, 4, 5].try_into().unwrap(),
                        vec![3, 4, 5].try_into().unwrap(),
                    ],
                );
            }
        }

        mod base4 {
            use super::*;

            #[test]
            fn test_15_15() {
                let mut iter = FirstCandidatesCombinationsIter::<Base4>::new(
                    15.try_into().unwrap(),
                    15.try_into().unwrap(),
                );

                assert_eq!(
                    iter.next().unwrap(),
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
                        .try_into()
                        .unwrap()
                );
                assert!(iter.next().is_none());
                assert!(iter.next().is_none());
            }

            #[test]
            fn test_1_16() {
                let iter = FirstCandidatesCombinationsIter::<Base4>::new(
                    1.try_into().unwrap(),
                    16.try_into().unwrap(),
                );

                itertools::assert_equal(
                    iter,
                    Candidates::<Base4>::all()
                        .iter()
                        .map(Candidates::with_single),
                );
            }
        }
    }

    mod candidates_combinations_iter {
        use std::{collections::BTreeSet, iter};

        use crate::{
            base::SudokuBase,
            cell::{Candidates, Value},
            test_util::{test_max_base5, test_max_base3, test_max_base4},
        };

        mod empty_candidates {
            use super::*;

            test_max_base5!({
                let empty: Candidates<Base> = Candidates::new();

                for k in Value::<Base>::all() {
                    // empty candidates never produce any combinations
                    itertools::assert_equal(empty.combinations(k), iter::empty());
                }
            });
        }

        mod single_candidate {
            use super::*;

            test_max_base5!({
                for single in Value::<Base>::all().map(Candidates::with_single) {
                    for k in Value::<Base>::all() {
                        if k.get() == 1 {
                            // a single candidate only produces a single combination if k == 1
                            itertools::assert_equal(single.combinations(k), iter::once(single));
                        } else {
                            // a single candidate never produces any combinations if k > 1
                            itertools::assert_equal(single.combinations(k), iter::empty());
                        }
                    }
                }
            });
        }

        mod three_candidates {
            use super::*;

            test_max_base5!({
                let three_candidates: Candidates<Base> = vec![1, 2, 4].try_into().unwrap();

                itertools::assert_equal(
                    three_candidates.combinations(1.try_into().unwrap()),
                    vec![
                        vec![1].try_into().unwrap(),
                        vec![2].try_into().unwrap(),
                        vec![4].try_into().unwrap(),
                    ],
                );

                itertools::assert_equal(
                    three_candidates.combinations(2.try_into().unwrap()),
                    vec![
                        vec![1, 2].try_into().unwrap(),
                        vec![1, 4].try_into().unwrap(),
                        vec![2, 4].try_into().unwrap(),
                    ],
                );
                itertools::assert_equal(
                    three_candidates.combinations(3.try_into().unwrap()),
                    iter::once(three_candidates),
                );
            });
        }

        mod all_candidates {
            use super::*;

            test_max_base5!({
                let all: Candidates<Base> = Candidates::all();

                // k == 1 produces all candidates
                itertools::assert_equal(
                    all.combinations(1.try_into().unwrap()),
                    all.iter().map(Candidates::with_single),
                );
            });
        }

        mod oracle {
            use super::*;

            fn assert_against_oracle<Base: SudokuBase>(
                candidates: Candidates<Base>,
                k: Value<Base>,
            ) {
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
                fn assert_set_equals<Base: SudokuBase>(
                    a: impl IntoIterator<Item = Candidates<Base>>,
                    b: impl IntoIterator<Item = Candidates<Base>>,
                ) {
                    let a: BTreeSet<_> = a.into_iter().collect();
                    let b: BTreeSet<_> = b.into_iter().collect();
                    assert_eq!(a, b);
                }

                let a = candidates.combinations(k);
                let b = combinations_itertools(candidates, k);
                assert_set_equals(a, b);
            }

            mod exhaustive {
                use super::*;

                // Exhaustive test up to Base3, since Base4 is too slow.
                test_max_base3!({
                    for candidates in Candidates::<Base>::iter_all_lexicographical() {
                        for k in Value::all() {
                            assert_against_oracle(candidates, k);
                        }
                    }
                });
            }

            mod empty_candidates {
                use super::*;

                test_max_base5!({
                    let empty: Candidates<Base> = Candidates::new();

                    for k in Value::<Base>::all() {
                        assert_against_oracle(empty, k);
                    }
                });
            }

            mod single_candidate {
                use super::*;

                test_max_base5!({
                    for single in Value::<Base>::all().map(Candidates::with_single) {
                        for k in Value::<Base>::all() {
                            assert_against_oracle(single, k);
                        }
                    }
                });
            }

            mod three_candidates {
                use super::*;

                test_max_base5!({
                    let three_candidates: Candidates<Base> = vec![1, 2, 4].try_into().unwrap();

                    for k in Value::all() {
                        assert_against_oracle(three_candidates, k);
                    }
                });
            }

            mod all_candidates {

                use super::*;

                test_max_base4!({
                    let all: Candidates<Base> = Candidates::all();

                    for k in Value::all() {
                        assert_against_oracle(all, k);
                    }
                });
            }

            mod all_k_1 {
                use super::*;

                test_max_base5!({
                    let all: Candidates<Base> = Candidates::all();

                    assert_against_oracle(all, 1.try_into().unwrap());
                });
            }
        }
    }
}
