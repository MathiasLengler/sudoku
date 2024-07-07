use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::Value;
use num::traits::{CheckedShl, ConstOne, WrappingAdd, WrappingNeg, WrappingShr, WrappingSub};
use num::{One, PrimInt, Unsigned, Zero};

// Reference: https://graphics.stanford.edu/%7Eseander/bithacks.html#NextBitPermutation
#[allow(clippy::many_single_char_names)]
fn next_permutation_u8(v: u8) -> Option<u8> {
    let t = v | (v.wrapping_sub(1));

    let a = !t;
    let b = a.wrapping_neg();
    let c = a & b;
    let d = c.wrapping_sub(1);
    let e = v.trailing_zeros();
    let f = e + 1;
    let g = d.wrapping_shr(f);
    let h = t.checked_add(1)?;
    Some(h | g)
}

fn next_permutation_num<
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

#[derive(Debug, Clone)]
struct FirstCandidatesCombinationsIter<Base: SudokuBase> {
    current: Candidates<Base>,
    n: Value<Base>,
    // TODO: fold into Option<Value<Base>> and benchmark
    is_finished: bool,
}

impl<Base: SudokuBase> FirstCandidatesCombinationsIter<Base> {
    /// Create a new iterator over all combinations of `k` candidates for the first `n` candidates.
    fn new(k: Value<Base>, n: Value<Base>) -> Self {
        let one = Base::CandidatesIntegral::ONE;
        if k > n {
            Self {
                current: Candidates::new(),
                n,
                is_finished: true,
            }
        } else {
            Self {
                current: Candidates::with_integral((one << k.get()) - one),
                n,
                is_finished: false,
            }
        }
    }
}

impl<Base: SudokuBase> Iterator for FirstCandidatesCombinationsIter<Base> {
    type Item = Candidates<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }
        let current = self.current;
        // println!("{current}");
        let next = next_permutation_num(current.integral())?;

        if (next & Base::CandidatesIntegral::ONE << (self.n.get())).is_zero() {
            let next = Candidates::with_integral(next);
            self.current = next;
            Some(current)
        } else {
            self.is_finished = true;
            Some(current)
        }
    }
}

/// An iterator over all combinations of `k` candidates contained in a given `Candidates`.
#[derive(Debug, Clone)]
pub struct CandidatesCombinationsIter<Base: SudokuBase> {
    candidates: Candidates<Base>,
    k: Value<Base>,
    current_combination: Candidates<Base>,
}

impl<Base: SudokuBase> CandidatesCombinationsIter<Base> {
    pub fn new(candidates: Candidates<Base>, k: Value<Base>) -> Self {
        todo!()
    }
}

impl<Base: SudokuBase> Iterator for CandidatesCombinationsIter<Base> {
    type Item = Candidates<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_permutation() {
        let expected = vec![
            0, 2, 4, 5, 8, 6, 9, 11, 16, 10, 12, 13, 17, 14, 19, 23, 32, 18, 20, 21, 24, 22, 25,
            27, 33, 26, 28, 29, 35, 30, 39, 47, 64, 34, 36, 37, 40, 38, 41, 43, 48, 42, 44, 45, 49,
            46, 51, 55, 65, 50, 52, 53, 56, 54, 57, 59, 67, 58, 60, 61, 71, 62, 79, 95, 128, 66,
            68, 69, 72, 70, 73, 75, 80, 74, 76, 77, 81, 78, 83, 87, 96, 82, 84, 85, 88, 86, 89, 91,
            97, 90, 92, 93, 99, 94, 103, 111, 129, 98, 100, 101, 104, 102, 105, 107, 112, 106, 108,
            109, 113, 110, 115, 119, 131, 114, 116, 117, 120, 118, 121, 123, 135, 122, 124, 125,
            143, 126, 159, 191, 0, 130, 132, 133, 136, 134, 137, 139, 144, 138, 140, 141, 145, 142,
            147, 151, 160, 146, 148, 149, 152, 150, 153, 155, 161, 154, 156, 157, 163, 158, 167,
            175, 192, 162, 164, 165, 168, 166, 169, 171, 176, 170, 172, 173, 177, 174, 179, 183,
            193, 178, 180, 181, 184, 182, 185, 187, 195, 186, 188, 189, 199, 190, 207, 223, 0, 194,
            196, 197, 200, 198, 201, 203, 208, 202, 204, 205, 209, 206, 211, 215, 224, 210, 212,
            213, 216, 214, 217, 219, 225, 218, 220, 221, 227, 222, 231, 239, 0, 226, 228, 229, 232,
            230, 233, 235, 240, 234, 236, 237, 241, 238, 243, 247, 0, 242, 244, 245, 248, 246, 249,
            251, 0, 250, 252, 253, 0, 254, 0, 0,
        ];

        itertools::assert_equal(
            (0..=u8::MAX).map(next_permutation_u8),
            expected
                .into_iter()
                .map(|i| if i == 0 { None } else { Some(i) }),
        );
        itertools::assert_equal(
            (0..=u8::MAX).map(next_permutation_u8),
            (0..=u8::MAX).map(next_permutation_num),
        );
    }

    mod first_candidates_combinations_iter {
        use std::iter;

        use crate::base::consts::*;

        use super::*;

        #[test]
        fn test_iter() {
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

            let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                3.try_into().unwrap(),
                3.try_into().unwrap(),
            );
            itertools::assert_equal(iter, iter::once(vec![1, 2, 3].try_into().unwrap()));

            let iter = FirstCandidatesCombinationsIter::<Base2>::new(
                4.try_into().unwrap(),
                3.try_into().unwrap(),
            );
            itertools::assert_equal(iter, iter::empty());

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
    }

    // #[test]
    // fn test_debug() {
    //     for v in 0..=u8::MAX {
    //         // let t = v | (v.wrapping_sub(1));
    //         // let res = (t.wrapping_add(1))
    //         //     | (((!t & (-(!t as i8)) as u8).wrapping_sub(1))
    //         //         .wrapping_shr((v.trailing_zeros() + 1)));

    //         let res = next_permutation_u8(v);
    //         let res2: u8 = next_permutation_num(v);
    //         // assert!(res == res2);

    //         println!("({v},{v:08b}) = ({res},{res:08b})");
    //     }
    // }

    #[test]
    fn test_debug_2() {
        let res = next_permutation_u8(192);
        dbg!(res);
    }
}
