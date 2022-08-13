use std::fmt::Debug;
use std::hash::Hash;
use std::mem::size_of;
use std::ops::*;

use generic_array::ArrayLength;
use typenum::{bit::B1, consts::*, Prod, Quot, Sub1, Sum};

pub mod consts {
    pub use typenum::consts::*;
}

pub type ArrayElement = u8;
type ArrayElementBitSize = U8;

/// (A + B - 1) / B;
type DivCeil<A, B> = Quot<Sub1<Sum<A, B>>, B>;
type SideLength<Base> = Prod<Base, Base>;
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

// New const/macro based API and Candidates BitArray

const fn base_to_side_length(base: usize) -> usize {
    base.pow(2)
}

const fn base_to_max_value(base: usize) -> usize {
    base_to_side_length(base)
}

const fn base_to_cell_count(base: usize) -> usize {
    base.pow(4)
}

const fn base_to_candidates_capacity<T>(base: usize) -> usize {
    let array_element_bit_size = size_of::<T>() * 8;
    let side_length = base_to_side_length(base);

    // div_ceil
    (side_length + array_element_bit_size - 1) / array_element_bit_size
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

    #[test]
    fn test_base_to_side_length() {
        assert_eq!(base_to_side_length(0), 0);
        assert_eq!(base_to_side_length(1), 1);
        assert_eq!(base_to_side_length(2), 4);
        assert_eq!(base_to_side_length(3), 9);
        assert_eq!(base_to_side_length(4), 16);
        assert_eq!(base_to_side_length(5), 25);
    }
    #[test]
    fn test_base_to_cell_count() {
        assert_eq!(base_to_cell_count(0), 0);
        assert_eq!(base_to_cell_count(1), 1);
        assert_eq!(base_to_cell_count(2), 16);
        assert_eq!(base_to_cell_count(3), 81);
        assert_eq!(base_to_cell_count(4), 256);
        assert_eq!(base_to_cell_count(5), 625);

        (0..=5).all(|base| base_to_side_length(base) == base_to_max_value(base));
    }
    #[test]
    fn test_base_to_candidates_capacity() {
        assert_eq!(base_to_candidates_capacity::<u8>(0), 0);
        assert_eq!(base_to_candidates_capacity::<u8>(1), 1);
        assert_eq!(base_to_candidates_capacity::<u8>(2), 1);
        assert_eq!(base_to_candidates_capacity::<u8>(3), 2);
        assert_eq!(base_to_candidates_capacity::<u8>(4), 2);
        assert_eq!(base_to_candidates_capacity::<u8>(5), 4);
    }
}
