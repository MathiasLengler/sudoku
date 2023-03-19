use std::fmt::{Binary, Debug, Display};
use std::hash::Hash;
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign, Shl};

use num::traits::{
    CheckedShl, CheckedShr, Unsigned, WrappingAdd, WrappingMul, WrappingShl, WrappingShr,
    WrappingSub,
};
use num::PrimInt;

use crate::cell::candidates_cell::CandidatesCell;
use consts::*;

pub mod consts {
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base1;
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base2;
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base3;
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base4;
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base5;

    pub use Base1 as U1;
    pub use Base2 as U2;
    pub use Base3 as U3;
    pub use Base4 as U4;
    pub use Base5 as U5;

    pub use Base5 as BaseMax;

    use crate::base::SudokuBase;

    pub const ALL_CELL_COUNTS: [u16; 5] = [
        Base1::CELL_COUNT,
        Base2::CELL_COUNT,
        Base3::CELL_COUNT,
        Base4::CELL_COUNT,
        Base5::CELL_COUNT,
    ];
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
         0,  0,  0,  0,  1,  1,  1,  1,  2,  2,  2,  2,  3,  3,  3,  3,
         0,  0,  0,  0,  1,  1,  1,  1,  2,  2,  2,  2,  3,  3,  3,  3,
         0,  0,  0,  0,  1,  1,  1,  1,  2,  2,  2,  2,  3,  3,  3,  3,
         0,  0,  0,  0,  1,  1,  1,  1,  2,  2,  2,  2,  3,  3,  3,  3, 
         4,  4,  4,  4,  5,  5,  5,  5,  6,  6,  6,  6,  7,  7,  7,  7,
         4,  4,  4,  4,  5,  5,  5,  5,  6,  6,  6,  6,  7,  7,  7,  7,
         4,  4,  4,  4,  5,  5,  5,  5,  6,  6,  6,  6,  7,  7,  7,  7,
         4,  4,  4,  4,  5,  5,  5,  5,  6,  6,  6,  6,  7,  7,  7,  7, 
         8,  8,  8,  8,  9,  9,  9,  9, 10, 10, 10, 10, 11, 11, 11, 11,
         8,  8,  8,  8,  9,  9,  9,  9, 10, 10, 10, 10, 11, 11, 11, 11,
         8,  8,  8,  8,  9,  9,  9,  9, 10, 10, 10, 10, 11, 11, 11, 11,
         8,  8,  8,  8,  9,  9,  9,  9, 10, 10, 10, 10, 11, 11, 11, 11,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
        12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14, 15, 15, 15, 15,
    ];
    #[rustfmt::skip]
    pub(super) static BASE_5: &[u8; base_to_cell_count(5) as usize] = &[
         0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,
         0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,
         0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,
         0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,
         0,  0,  0,  0,  0,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,
         5,  5,  5,  5,  5,  6,  6,  6,  6,  6,  7,  7,  7,  7,  7,  8,  8,  8,  8,  8,  9,  9,  9,  9,  9,
         5,  5,  5,  5,  5,  6,  6,  6,  6,  6,  7,  7,  7,  7,  7,  8,  8,  8,  8,  8,  9,  9,  9,  9,  9,
         5,  5,  5,  5,  5,  6,  6,  6,  6,  6,  7,  7,  7,  7,  7,  8,  8,  8,  8,  8,  9,  9,  9,  9,  9,
         5,  5,  5,  5,  5,  6,  6,  6,  6,  6,  7,  7,  7,  7,  7,  8,  8,  8,  8,  8,  9,  9,  9,  9,  9,
         5,  5,  5,  5,  5,  6,  6,  6,  6,  6,  7,  7,  7,  7,  7,  8,  8,  8,  8,  8,  9,  9,  9,  9,  9,
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

/// A size of a square, standard sudoku.
///
/// The base of sudoku equals the
///
/// Implementations for bases `1..=5` are provided by this crate.
///
/// # Safety
/// This crate makes assumptions about the correct implementation of this trait.
/// An incorrect implementation could result in undefined behaviour.
pub unsafe trait SudokuBase
where
    Self: Ord + Hash + Clone + Copy + Debug + Default + 'static,
{
    // TODO: evaluate `as` casting of constants
    /// The side length of a sudoku block.
    ///
    /// # Examples
    /// - `4x4`: `2`
    /// - `9x9`: `3`
    /// - `16x16`: `4`
    /// - `25x25`: `5`
    const BASE: u8;
    /// The side length of the complete sudoku. Equals this size of a row or column.
    /// Must equal `MAX_VALUE`.
    const SIDE_LENGTH: u8;
    /// The max value a value can be set to.
    /// Must equal `SIDE_LENGTH`.
    const MAX_VALUE: u8;
    /// The total cell count of the sudoku.
    const CELL_COUNT: u16;

    /// Get the the block index for a specific cell index.
    fn cell_index_to_block_index(cell_index: u16) -> u8;

    /// Bit field type for candidates storage.
    type CandidatesIntegral: Copy
        + Clone
        + Debug
        + Default
        + Display
        + Binary
        + Hash
        // Generic bit twiddling
        + PrimInt
        + CheckedShl
        + CheckedShr
        + Unsigned
        + WrappingAdd
        + WrappingMul
        + WrappingShl
        + WrappingShr
        + WrappingSub
        + BitXorAssign
        + BitOrAssign
        + BitAndAssign
        + Shl<u8, Output = Self::CandidatesIntegral>;

    /// Data structure for `backtracking_bitset::Solver`
    type CandidatesCells: AsRef<[CandidatesCell<Self>]>
        + AsMut<[CandidatesCell<Self>]>
        + Clone
        + Debug
        + Default;
}

macro_rules! impl_sudoku_base {
    ($($type_num:ty,$base_u8:expr,$type_integral:ty,$CELL_INDEX_TO_BLOCK_INDEX:expr;)+) => {
        $(
unsafe impl SudokuBase for $type_num {
    const BASE: u8 = $base_u8;
    const SIDE_LENGTH: u8 = base_to_side_length(Self::BASE);
    const MAX_VALUE: u8 = base_to_max_value(Self::BASE);
    const CELL_COUNT: u16 = base_to_cell_count(Self::BASE);

    fn cell_index_to_block_index(cell_index: u16) -> u8 {
        $CELL_INDEX_TO_BLOCK_INDEX[usize::from(cell_index)]
    }

    type CandidatesIntegral = $type_integral;

    type CandidatesCells = [CandidatesCell<Self>; Self::SIDE_LENGTH as usize];
}
        )+
    };
}

// All sudoku bases supported by DynamicSudoku, and U1 for testing.
impl_sudoku_base!(
    Base1, 1, u8, cell_index_to_block_index::BASE_1;
    Base2, 2, u8, cell_index_to_block_index::BASE_2;
    Base3, 3, u16, cell_index_to_block_index::BASE_3;
    Base4, 4, u16, cell_index_to_block_index::BASE_4;
    Base5, 5, u32, cell_index_to_block_index::BASE_5;
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
        use std::mem::size_of;

        // MAX_VALUE must be representable at the highest bit position.
        assert!(size_of::<Base::CandidatesIntegral>() * 8 >= usize::from(Base::MAX_VALUE))
    }

    #[test]
    fn test_base_2() {
        type Base = U2;

        assert_eq!(Base::BASE, 2);
        assert_eq!(Base::SIDE_LENGTH, 4);
        assert_eq!(Base::MAX_VALUE, 4);
        assert_eq!(Base::CELL_COUNT, 16);

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
