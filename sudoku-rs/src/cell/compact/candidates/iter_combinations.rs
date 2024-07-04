use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::Value;
use num::traits::{CheckedShl, WrappingSub};
use num::{One, PrimInt, Zero};
#[derive(Debug, Clone)]
struct BitPermutationIter<Base: SudokuBase> {
    // current permutation of bits
    v: Base::CandidatesIntegral,
}

impl<Base: SudokuBase> BitPermutationIter<Base> {
    fn new(v: Base::CandidatesIntegral) -> Self {
        Self { v }
    }
}

// impl<Base: SudokuBase> Iterator for BitPermutationIter<Base> {
//     type Item = Base::CandidatesIntegral;

//     // Reference: https://graphics.stanford.edu/%7Eseander/bithacks.html#NextBitPermutation
//     fn next(&mut self) -> Option<Self::Item> {
//         let one = Base::CandidatesIntegral::one();

//         let v = self.v;
//         let t = v | (v - one); // t gets v's least significant 0 bits set to 1
//                                // Next set to 1 the most significant bit to change,
//                                // set to 0 the least significant ones, and add the necessary 1 bits.
//         w = (t + one) | (((!t & -!t) - 1) >> (__builtin_ctz(v) + 1));
//     }
// }

/// An iterator over all combinations of `k` candidates contained in a given `Candidates`.
#[derive(Debug, Clone)]
pub struct CandidatesCombinationsIter<Base: SudokuBase> {
    candidates: Candidates<Base>,
    k: Value<Base>,
    current_combination: Candidates<Base>,
}

// impl<Base: SudokuBase> Iterator for CandidatesCombinationsIter<Base> {
//     type Item = Candidates<Base>;

//     fn next(&mut self) -> Option<Self::Item> {}
// }

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: translate cursed bit twiddling expression into equivalent Rust code
    //  Reference: https://graphics.stanford.edu/%7Eseander/bithacks.html#NextBitPermutation
    //  compare output with C compiler for all u8 values
    //  only then, tranlate into generic `num` using `num_traits::cast::AsPrimitive` and friends
    #[test]
    fn test_debug() {
        for v in 0..=u8::MAX {
            // let t = v | (v.wrapping_sub(1));
            // let res = (t.wrapping_add(1))
            //     | (((!t & (-(!t as i8)) as u8).wrapping_sub(1))
            //         .wrapping_shr((v.trailing_zeros() + 1)));

            let t = v;

            // In C: ((~t & -~t) - 1)
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let res = (!t & ((!t as i8).wrapping_neg()) as u8).wrapping_sub(1);
            println!("{v} = {res}");
        }
    }
}
