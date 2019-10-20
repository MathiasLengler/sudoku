use super::*;

type DivCeil<A, B> = Quot<Sub1<Sum<A, B>>, B>;
// (A + B - 1) / B;
type SideLength<Base> = Prod<Base, Base>;
type CandidatesCapacity<Base> = DivCeil<SideLength<Base>, U8>;
type CellCount<Base> = Prod<SideLength<Base>, SideLength<Base>>;

pub trait SudokuBase: ArrayLength<u8> + Ord + Hash + Clone + Debug {
    type SideLength: ArrayLength<u8> + Ord + Hash + Clone + Debug;
    type MaxValue: ArrayLength<u8> + Ord + Hash + Clone + Debug;
    type CandidatesCapacity: ArrayLength<u8> + Ord + Hash + Clone + Debug;
    type CellCount: ArrayLength<u8> + Ord + Hash + Clone + Debug;
}

impl<Base> SudokuBase for Base
where
    Base: ArrayLength<u8> + Ord + Hash + Clone + Debug,
    SideLength<Base>: ArrayLength<u8> + Ord + Hash + Clone + Debug,
    CandidatesCapacity<Base>: ArrayLength<u8> + Ord + Hash + Clone + Debug,
    CellCount<Base>: ArrayLength<u8> + Ord + Hash + Clone + Debug,
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
