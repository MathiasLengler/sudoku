use std::fmt::Debug;
use std::hash::Hash;
use std::ops::*;

use generic_array::ArrayLength;
use typenum::{bit::B1, consts::*, Prod, Quot, Sub1, Sum};

pub mod consts {
    pub use typenum::consts::*;
}

type DivCeil<A, B> = Quot<Sub1<Sum<A, B>>, B>;
// (A + B - 1) / B;
type SideLength<Base> = Prod<Base, Base>;
type CandidatesCapacity<Base> = DivCeil<SideLength<Base>, U8>;
type CellCount<Base> = Prod<SideLength<Base>, SideLength<Base>>;

/// Trait "alias" for Sudoku Base typenum.
/// Users of SudokuBase only need to specify this bound:
///
/// `<Base: SudokuBase>`
///
/// and can access computed typenum values for a given base via the associated types.
///
/// Without this trait users would have to repeat all bounds on every generic declaration.
///
/// This is a workaround for [RFC 2089: Implied bounds](https://github.com/rust-lang/rust/issues/44491).
pub trait SudokuBase
where
    Self: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug,
{
    type SideLength: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug;
    type MaxValue: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug;
    type CandidatesCapacity: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug;
    type CellCount: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug;
}

impl<Base> SudokuBase for Base
where
    Base: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug,
    SideLength<Base>: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug,
    CandidatesCapacity<Base>: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug,
    CellCount<Base>: ArrayLength<u8> + Ord + Hash + Clone + Copy + Debug,
    Base: Mul<Base>,
    SideLength<Base>: Add<U8>,
    Sum<SideLength<Base>, U8>: Sub<B1>,
    Sub1<Sum<SideLength<Base>, U8>>: Div<U8>,
    SideLength<Base>: Mul<Base>,
    SideLength<Base>: Mul<SideLength<Base>>,
{
    type SideLength = SideLength<Base>;
    type MaxValue = SideLength<Base>;
    type CandidatesCapacity = CandidatesCapacity<Base>;
    type CellCount = CellCount<Base>;
}

#[cfg(test)]
mod tests {
    use typenum::Unsigned;

    use super::*;

    #[test]
    fn test_base_constraints() {
        type SideLength<Base> = <Base as SudokuBase>::SideLength;
        type MaxValue<Base> = <Base as SudokuBase>::MaxValue;
        type CandidatesCapacity<Base> = <Base as SudokuBase>::CandidatesCapacity;
        type CellCount<Base> = <Base as SudokuBase>::CellCount;

        assert_eq!(SideLength::<U0>::to_u8(), 0);
        assert_eq!(SideLength::<U1>::to_u8(), 1);
        assert_eq!(SideLength::<U2>::to_u8(), 4);
        assert_eq!(SideLength::<U3>::to_u8(), 9);
        assert_eq!(SideLength::<U4>::to_u8(), 16);
        assert_eq!(MaxValue::<U0>::to_u8(), 0);
        assert_eq!(MaxValue::<U1>::to_u8(), 1);
        assert_eq!(MaxValue::<U2>::to_u8(), 4);
        assert_eq!(MaxValue::<U3>::to_u8(), 9);
        assert_eq!(MaxValue::<U4>::to_u8(), 16);
        assert_eq!(CellCount::<U0>::to_u8(), 0);
        assert_eq!(CellCount::<U1>::to_u8(), 1);
        assert_eq!(CellCount::<U2>::to_u8(), 16);
        assert_eq!(CellCount::<U3>::to_u8(), 81);
        assert_eq!(CellCount::<U4>::to_u16(), 256);
        assert_eq!(CandidatesCapacity::<U0>::to_u8(), 0);
        assert_eq!(CandidatesCapacity::<U1>::to_u8(), 1);
        assert_eq!(CandidatesCapacity::<U2>::to_u8(), 1);
        assert_eq!(CandidatesCapacity::<U3>::to_u8(), 2);
        assert_eq!(CandidatesCapacity::<U4>::to_u8(), 2);
    }
}
