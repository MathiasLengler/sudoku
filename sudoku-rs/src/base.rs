use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::mem::size_of;

use bitvec::store::BitStore;
use bitvec::view::BitViewSized;
use funty::Integral;
use typenum::{
    consts::{U1, U2, U3, U4, U5},
    Unsigned,
};

use crate::cell::candidates_cell::CandidatesCell;

pub mod consts {
    pub use typenum::consts::{U1, U2, U3, U4, U5};
}

pub type ArrayElement = u8;

const fn base_to_side_length(base: u8) -> u8 {
    base.pow(2)
}

const fn base_to_max_value(base: u8) -> u8 {
    base_to_side_length(base)
}

const fn base_to_cell_count(base: u8) -> u16 {
    (base as u16).pow(4)
}

const fn base_to_candidates_capacity<T>(base: u8) -> usize {
    let array_element_bit_size = size_of::<T>() * 8;
    let max_value = base_to_max_value(base) as usize;

    // div_ceil
    (max_value + array_element_bit_size - 1) / array_element_bit_size
}

mod cell_index_to_block_index {
    use super::*;

    pub(super) static BASE_1: &[u8; base_to_cell_count(1) as usize] = &[0];
    pub(super) static BASE_2: &[u8; base_to_cell_count(2) as usize] = &[
        0, 0, 1, 1, //
        0, 0, 1, 1, //
        2, 2, 3, 3, //
        2, 2, 3, 3, //
    ];
    pub(super) static BASE_3: &[u8; base_to_cell_count(3) as usize] = &[
        0, 0, 0, 1, 1, 1, 2, 2, 2, //
        0, 0, 0, 1, 1, 1, 2, 2, 2, //
        0, 0, 0, 1, 1, 1, 2, 2, 2, //
        3, 3, 3, 4, 4, 4, 5, 5, 5, //
        3, 3, 3, 4, 4, 4, 5, 5, 5, //
        3, 3, 3, 4, 4, 4, 5, 5, 5, //
        6, 6, 6, 7, 7, 7, 8, 8, 8, //
        6, 6, 6, 7, 7, 7, 8, 8, 8, //
        6, 6, 6, 7, 7, 7, 8, 8, 8, //
    ];
    #[rustfmt::skip]
    pub(super) static BASE_4: &[u8; base_to_cell_count(4) as usize] = &[
        0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
        0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
        0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
        0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 
        4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7,
        4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7,
        4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7,
        4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 
        8, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11,
        8, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11,
        8, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11,
        8, 8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
    ];
    #[rustfmt::skip]
    pub(super) static BASE_5: &[u8; base_to_cell_count(5) as usize] = &[
        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
        0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
        5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9,
        5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9,
        5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9,
        5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9,
        5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9,
        10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14,
        10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14,
        10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14,
        10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14,
        10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14,
        15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19,
        15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19,
        15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19,
        15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19,
        15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19,
        20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24,
        20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24,
        20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24,
        20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24,
        20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24
    ];
}

// TODO: evaluate `as` casting of constants
pub trait SudokuBase
where
    Self: Ord + Hash + Clone + Copy + Debug + Default,
{
    const BASE: u8;
    const SIDE_LENGTH: u8;
    const MAX_VALUE: u8;
    const CELL_COUNT: u16;
    const CANDIDATES_ARRAY_CAPACITY: usize;

    fn cell_index_to_block_index(cell_index: u16) -> u8;

    type CandidatesArrayElement: BitStore;
    type CandidatesArray: BitViewSized + Ord + Hash + Copy + Clone + Debug + Default;
    type CandidatesIntegral: BitStore + Integral + Copy + Clone + Debug + Default + Display;

    type CandidatesCells: AsRef<[CandidatesCell<Self>]>
        + AsMut<[CandidatesCell<Self>]>
        + Clone
        + Debug
        + Default;
}

macro_rules! impl_sudoku_base {
    ($($type_num:ty,$type_integral:ty,$CELL_INDEX_TO_BLOCK_INDEX:expr;)+) => {
        $(
impl SudokuBase for $type_num {
    const BASE: u8 = Self::U8;
    const SIDE_LENGTH: u8 = base_to_side_length(Self::BASE);
    const MAX_VALUE: u8 = base_to_max_value(Self::BASE);
    const CELL_COUNT: u16 = base_to_cell_count(Self::BASE);
    const CANDIDATES_ARRAY_CAPACITY: usize =
        base_to_candidates_capacity::<Self::CandidatesArrayElement>(Self::BASE);

    fn cell_index_to_block_index(cell_index: u16) -> u8 {
        $CELL_INDEX_TO_BLOCK_INDEX[usize::from(cell_index)]
    }

    type CandidatesArrayElement = u8;
    type CandidatesArray = [Self::CandidatesArrayElement; Self::CANDIDATES_ARRAY_CAPACITY];
    type CandidatesIntegral = $type_integral;

    type CandidatesCells = [CandidatesCell<Self>; Self::MAX_VALUE as usize];
}
        )+
    };
}

// All sudoku bases supported by DynamicSudoku, and U1 for testing.
impl_sudoku_base!(
    U1, u8, cell_index_to_block_index::BASE_1;
    U2, u8, cell_index_to_block_index::BASE_2;
    U3, u16, cell_index_to_block_index::BASE_3;
    U4, u16, cell_index_to_block_index::BASE_4;
    U5, u32, cell_index_to_block_index::BASE_5;
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_to_side_length() {
        assert_eq!(base_to_side_length(0), 0);
        assert_eq!(base_to_side_length(1), 1);
        assert_eq!(base_to_side_length(2), 4);
        assert_eq!(base_to_side_length(3), 9);
        assert_eq!(base_to_side_length(4), 16);
        assert_eq!(base_to_side_length(5), 25);

        assert!((0..=5).all(|base| base_to_side_length(base) == base_to_max_value(base)));
    }
    #[test]
    fn test_base_to_cell_count() {
        assert_eq!(base_to_cell_count(0), 0);
        assert_eq!(base_to_cell_count(1), 1);
        assert_eq!(base_to_cell_count(2), 16);
        assert_eq!(base_to_cell_count(3), 81);
        assert_eq!(base_to_cell_count(4), 256);
        assert_eq!(base_to_cell_count(5), 625);
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

    // Fork of https://docs.rs/type-equals/0.1.0/type_equals/
    trait TypeEquals {
        type Other: ?Sized;
    }

    impl<T: ?Sized> TypeEquals for T {
        type Other = Self;
    }

    fn assert_type_equals<L, R>()
    where
        L: TypeEquals<Other = R>,
    {
    }

    fn assert_base_invariants<Base: SudokuBase>() {
        assert_eq!(
            size_of::<Base::CandidatesArray>(),
            size_of::<Base::CandidatesIntegral>()
        );

        // MAX_VALUE must be representable at the highest bit position.
        assert!(size_of::<Base::CandidatesArray>() * 8 >= usize::from(Base::MAX_VALUE))
    }

    #[test]
    fn test_base_2() {
        type Base = U2;

        assert_eq!(Base::BASE, 2);
        assert_eq!(Base::SIDE_LENGTH, 4);
        assert_eq!(Base::MAX_VALUE, 4);
        assert_eq!(Base::CELL_COUNT, 16);
        assert_eq!(Base::CANDIDATES_ARRAY_CAPACITY, 1);

        assert_type_equals::<<Base as SudokuBase>::CandidatesArrayElement, u8>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesArray, [u8; 1]>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u8>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_base_3() {
        type Base = U3;

        assert_eq!(Base::BASE, 3);
        assert_eq!(Base::SIDE_LENGTH, 9);
        assert_eq!(Base::MAX_VALUE, 9);
        assert_eq!(Base::CELL_COUNT, 81);
        assert_eq!(Base::CANDIDATES_ARRAY_CAPACITY, 2);

        assert_type_equals::<<Base as SudokuBase>::CandidatesArrayElement, u8>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesArray, [u8; 2]>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u16>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_base_4() {
        type Base = U4;

        assert_eq!(Base::BASE, 4);
        assert_eq!(Base::SIDE_LENGTH, 16);
        assert_eq!(Base::MAX_VALUE, 16);
        assert_eq!(Base::CELL_COUNT, 256);
        assert_eq!(Base::CANDIDATES_ARRAY_CAPACITY, 2);

        assert_type_equals::<<Base as SudokuBase>::CandidatesArrayElement, u8>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesArray, [u8; 2]>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u16>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_base_5() {
        type Base = U5;

        assert_eq!(Base::BASE, 5);
        assert_eq!(Base::SIDE_LENGTH, 25);
        assert_eq!(Base::MAX_VALUE, 25);
        assert_eq!(Base::CELL_COUNT, 625);
        assert_eq!(Base::CANDIDATES_ARRAY_CAPACITY, 4);

        assert_type_equals::<<Base as SudokuBase>::CandidatesArrayElement, u8>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesArray, [u8; 4]>();
        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u32>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_cell_index_to_block_index() {
        fn generate_cell_index_to_block_index(base: u8) -> Vec<u8> {
            use std::iter::repeat;
            let base_usize = usize::from(base);
            let block_row_cell_count = base_usize * base_usize * base_usize;
            (0..base)
                .flat_map(|row| {
                    let starting_block_index = row * base;
                    (starting_block_index..(starting_block_index + base))
                        .flat_map(|i| repeat(i).take(base_usize))
                        .cycle()
                        .take(block_row_cell_count)
                })
                .collect::<Vec<_>>()
        }

        assert_eq!(
            cell_index_to_block_index::BASE_1,
            generate_cell_index_to_block_index(1).as_slice()
        );
        assert_eq!(
            cell_index_to_block_index::BASE_2,
            generate_cell_index_to_block_index(2).as_slice()
        );
        assert_eq!(
            cell_index_to_block_index::BASE_3,
            generate_cell_index_to_block_index(3).as_slice()
        );
        assert_eq!(
            cell_index_to_block_index::BASE_4,
            generate_cell_index_to_block_index(4).as_slice()
        );
        assert_eq!(
            cell_index_to_block_index::BASE_5,
            generate_cell_index_to_block_index(5).as_slice()
        );
    }
}
