use std::fmt::Debug;
use std::hash::Hash;
use std::ops::*;

use generic_array::ArrayLength;
use typenum::{bit::B1, consts::*, Prod, Quot, Sub1, Sum};

pub mod consts {
    pub use typenum::consts::*;
}

pub type ArrayElement = u8;
type ArrayElementBitSize = U8;

// TODO: compare with: fixed-bitset
type DivCeil<A, B> = Quot<Sub1<Sum<A, B>>, B>;
// (A + B - 1) / B;
type SideLength<Base> = Prod<Base, Base>;
// TODO: as well as storage size (U8)
type CandidatesCapacity<Base> = DivCeil<SideLength<Base>, ArrayElementBitSize>;
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
    Self: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default,
{
    type SideLength: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default;
    type MaxValue: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default;
    type CandidatesCapacity: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default;
    type CellCount: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default;
}

impl<Base> SudokuBase for Base
where
    Base: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default,
    SideLength<Base>: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default,
    CandidatesCapacity<Base>:
        ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default,
    CellCount<Base>: ArrayLength<ArrayElement> + Ord + Hash + Clone + Copy + Debug + Default,
    Base: Mul<Base>,
    SideLength<Base>: Add<ArrayElementBitSize>,
    Sum<SideLength<Base>, ArrayElementBitSize>: Sub<B1>,
    Sub1<Sum<SideLength<Base>, ArrayElementBitSize>>: Div<ArrayElementBitSize>,
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
