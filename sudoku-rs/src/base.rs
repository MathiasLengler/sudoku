use std::fmt::{Binary, Debug, Display};
use std::hash::Hash;
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign, Shl};

use num::traits::{
    CheckedShl, CheckedShr, Unsigned, WrappingAdd, WrappingMul, WrappingShl, WrappingShr,
    WrappingSub,
};
use num::PrimInt;

use consts::*;

use crate::cell::candidates_cell::CandidatesCell;
use crate::error::Error;
use crate::position::Coordinate;
use crate::position::Position;
use crate::unsafe_utils::get_unchecked;

pub mod consts {
    pub use Base5 as BaseMax;

    use crate::base::SudokuBase;

    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base2;
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base3;
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base4;
    #[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Base5;

    pub const ALL_CELL_COUNTS: [u16; 4] = [
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

const fn base_to_binary_fixed_candidates_line_cell_chars(base: u8) -> usize {
    match base {
        2 => 1,
        3 => 2,
        4 => 4,
        5 => 6,
        _ => panic!("Unexpected base"),
    }
}

mod cell_index_to_block_index {
    //! # Safety
    //! Each `array` must fulfill the following properties for its respective `Base`:
    //! - For all values (`block_index`): `block_index < Base::SIDE_LENGTH`
    //! - `array.len() == Base::CELL_COUNT`
    use super::*;

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

mod block_index_to_top_left_cell_index {
    //! # Safety
    //! Each `array` must fulfill the following properties for its respective `Base`:
    //! - For all values (`cell_index`): `cell_index < Base::CELL_COUNT`
    //! - `array.len() == Base::SIDE_LENGTH`
    use super::*;

    pub(super) static BASE_2: [u16; base_to_side_length(2) as usize] = [0, 2, 8, 10];
    pub(super) static BASE_3: [u16; base_to_side_length(3) as usize] =
        [0, 3, 6, 27, 30, 33, 54, 57, 60];
    pub(super) static BASE_4: [u16; base_to_side_length(4) as usize] = [
        0, 4, 8, 12, 64, 68, 72, 76, 128, 132, 136, 140, 192, 196, 200, 204,
    ];
    pub(super) static BASE_5: [u16; base_to_side_length(5) as usize] = [
        0, 5, 10, 15, 20, 125, 130, 135, 140, 145, 250, 255, 260, 265, 270, 375, 380, 385, 390,
        395, 500, 505, 510, 515, 520,
    ];
}

/// A size of a square, standard sudoku.
///
/// Implementations for bases `2..=5` are provided by this crate.
///
/// # Safety
/// This crate makes assumptions about the correct implementation of this trait.
/// An incorrect implementation could result in undefined behaviour.
pub unsafe trait SudokuBase
where
    Self: Ord + Hash + Clone + Copy + Debug + Default + 'static,
{
    // TODO: evaluate `as` casting of constants
    /// The side length of a sudoku block. Must be non-zero.
    ///
    /// # Safety
    /// Must be non-zero.
    ///
    /// # Examples
    /// - `4x4`: `2`
    /// - `9x9`: `3`
    /// - `16x16`: `4`
    /// - `25x25`: `5`
    const BASE: u8;
    /// The side length of the complete sudoku. Equals this size of a row or column.
    ///
    /// # Safety
    /// - must equal `BASE.pow(2)`
    /// - must equal `MAX_VALUE`
    const SIDE_LENGTH: u8;
    /// The max value a value can be set to.
    ///
    /// # Safety
    /// - must equal `BASE.pow(2)`
    /// - must equal `SIDE_LENGTH`
    const MAX_VALUE: u8;
    /// The total cell count of the sudoku.
    ///
    /// # Safety
    /// - must equal `(base as u16).pow(4)`
    const CELL_COUNT: u16;

    /// Used by `BinaryFixedCandidatesLine`
    const BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS: usize;

    /// For a given cell position, returns the coordinate of the block it is contained in.
    fn pos_to_block(pos: Position<Self>) -> Coordinate<Self>;

    /// For a given block, returns the position of the top left cell in this block.
    fn block_to_top_left_pos(block: Coordinate<Self>) -> Position<Self>;

    /// Bit field type for candidates storage.
    ///
    /// # Safety
    ///
    /// `MAX_VALUE` must be representable at the highest bit position,
    /// e.g. the size of the unsigned primitive must be equal to or greater than `MAX_VALUE`.
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
        + Shl<u8, Output = Self::CandidatesIntegral>
        + Into<u32>
        + TryFrom<u32, Error = Self::CandidatesIntegralTryFromU32Error>;

    type CandidatesIntegralTryFromU32Error: Into<Error> + Debug;

    /// Data structure for `backtracking_bitset::Solver`.
    ///
    /// # Safety
    ///
    /// The length of the array must equal `Base::SIDE_LENGTH`.
    type CandidatesCells: AsRef<[CandidatesCell<Self>]>
        + AsMut<[CandidatesCell<Self>]>
        + Clone
        + Debug
        + Default;
}

macro_rules! impl_sudoku_base {
    ($($type_num:ty,$base_u8:expr,$type_integral:ty,$CELL_INDEX_TO_BLOCK_INDEX:expr,$BLOCK_INDEX_TO_TOP_LEFT_CELL_INDEX:expr;)+) => {
        $(
// Safety: this private macro is only instantiated below and the correctness of the generated impls is tested.
unsafe impl SudokuBase for $type_num {
    const BASE: u8 = $base_u8;
    const SIDE_LENGTH: u8 = base_to_side_length(Self::BASE);
    const MAX_VALUE: u8 = base_to_max_value(Self::BASE);
    const CELL_COUNT: u16 = base_to_cell_count(Self::BASE);
    const BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS: usize = base_to_binary_fixed_candidates_line_cell_chars(Self::BASE);

    fn pos_to_block(pos: Position<Self>) -> Coordinate<Self> {
        let cell_index = usize::from(pos.cell_index());
        // Safety: relies on invariants:
        // - module `cell_index_to_block_index`: `CELL_INDEX_TO_BLOCK_INDEX.len() == Base::CELL_COUNT`
        // - `Position::<Base>::cell_index`: `pos.cell_index < Base::CELL_COUNT`
        // Therefore the index remains in-bounds.
        let block_index = unsafe { get_unchecked($CELL_INDEX_TO_BLOCK_INDEX.as_slice(), cell_index) };
        // Safety: `block_index` remains in-bounds, guaranteed by module `cell_index_to_block_index`
        unsafe { Coordinate::new_unchecked(*block_index) }
    }

    fn block_to_top_left_pos(block: Coordinate<Self>) -> Position<Self> {
        let index = usize::from(block.get());
        // Safety: relies on invariants:
        // - module `block_index_to_top_left_cell_index`: `BLOCK_TO_TOP_LEFT_CELL_INDEX.len() == Base::SIDE_LENGTH`
        // - `Coordinate::<Base>::get`: `coordinate < Base::SIDE_LENGTH`
        // Therefore the index remains in-bounds.
        let cell_index = unsafe { get_unchecked($BLOCK_INDEX_TO_TOP_LEFT_CELL_INDEX.as_slice(), index) };

        // Safety: `cell_index` remains in-bounds, guaranteed by module `block_to_top_left_cell_index`
        unsafe { Position::new_unchecked(*cell_index) }
    }

    type CandidatesIntegral = $type_integral;

    type CandidatesIntegralTryFromU32Error = <$type_integral as TryFrom<u32>>::Error;

    type CandidatesCells = [CandidatesCell<Self>; Self::SIDE_LENGTH as usize];
}
        )+
    };
}

// All sudoku bases supported by DynamicSudoku, and U1 for testing.
impl_sudoku_base!(
    Base2, 2, u8, cell_index_to_block_index::BASE_2, block_index_to_top_left_cell_index::BASE_2;
    Base3, 3, u16, cell_index_to_block_index::BASE_3, block_index_to_top_left_cell_index::BASE_3;
    Base4, 4, u16, cell_index_to_block_index::BASE_4, block_index_to_top_left_cell_index::BASE_4;
    Base5, 5, u32, cell_index_to_block_index::BASE_5, block_index_to_top_left_cell_index::BASE_5;
);

#[cfg(test)]
mod tests {
    use crate::position::DynamicPosition;

    use super::*;

    #[test]
    fn test_base_to_side_length_or_max_value() {
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

        // Safety invariant of Base::BASE
        assert_ne!(Base::BASE, 0);

        // Safety invariants of Base::SIDE_LENGTH/Base::MAX_VALUE
        assert_eq!(Base::SIDE_LENGTH, Base::MAX_VALUE);
        assert_eq!(Base::SIDE_LENGTH, base_to_side_length(Base::BASE));
        assert_eq!(Base::MAX_VALUE, base_to_max_value(Base::BASE));

        // Safety invariant of Base::CELL_COUNT
        assert_eq!(Base::CELL_COUNT, base_to_cell_count(Base::BASE));

        // Safety invariant of Base::CandidatesIntegral
        // MAX_VALUE must be representable at the highest bit position.
        assert!(size_of::<Base::CandidatesIntegral>() * 8 >= usize::from(Base::MAX_VALUE));
        // Safety invariant of Base::CandidatesCells
        let mut candidates_cells = <Base as SudokuBase>::CandidatesCells::default();
        assert_eq!(
            candidates_cells.as_ref().len(),
            usize::from(Base::SIDE_LENGTH)
        );
        assert_eq!(
            candidates_cells.as_mut().len(),
            usize::from(Base::SIDE_LENGTH)
        );
    }

    #[test]
    fn test_base_2() {
        type Base = Base2;

        assert_eq!(Base::BASE, 2);
        assert_eq!(Base::SIDE_LENGTH, 4);
        assert_eq!(Base::MAX_VALUE, 4);
        assert_eq!(Base::CELL_COUNT, 16);
        assert_eq!(Base::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS, 1);

        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u8>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_base_3() {
        type Base = Base3;

        assert_eq!(Base::BASE, 3);
        assert_eq!(Base::SIDE_LENGTH, 9);
        assert_eq!(Base::MAX_VALUE, 9);
        assert_eq!(Base::CELL_COUNT, 81);
        assert_eq!(Base::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS, 2);

        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u16>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_base_4() {
        type Base = Base4;

        assert_eq!(Base::BASE, 4);
        assert_eq!(Base::SIDE_LENGTH, 16);
        assert_eq!(Base::MAX_VALUE, 16);
        assert_eq!(Base::CELL_COUNT, 256);
        assert_eq!(Base::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS, 4);

        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u16>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_base_5() {
        type Base = Base5;

        assert_eq!(Base::BASE, 5);
        assert_eq!(Base::SIDE_LENGTH, 25);
        assert_eq!(Base::MAX_VALUE, 25);
        assert_eq!(Base::CELL_COUNT, 625);
        assert_eq!(Base::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS, 6);

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

    #[test]
    fn test_pos_to_block() {
        let base2: Vec<_> = Position::<Base2>::all()
            .map(SudokuBase::pos_to_block)
            .map(Coordinate::get)
            .collect();
        assert_eq!(cell_index_to_block_index::BASE_2, base2.as_slice());

        let base3: Vec<_> = Position::<Base3>::all()
            .map(SudokuBase::pos_to_block)
            .map(Coordinate::get)
            .collect();
        assert_eq!(cell_index_to_block_index::BASE_3, base3.as_slice());

        let base4: Vec<_> = Position::<Base4>::all()
            .map(SudokuBase::pos_to_block)
            .map(Coordinate::get)
            .collect();
        assert_eq!(cell_index_to_block_index::BASE_4, base4.as_slice());

        let base5: Vec<_> = Position::<Base5>::all()
            .map(SudokuBase::pos_to_block)
            .map(Coordinate::get)
            .collect();
        assert_eq!(cell_index_to_block_index::BASE_5, base5.as_slice());
    }

    #[test]
    fn test_mod_block_to_top_left_pos() {
        fn generate_block_to_top_left_cell_index(base: u8) -> Vec<u16> {
            use num::Integer;

            let side_length = base_to_side_length(base);
            (0..side_length)
                .map(|block_index| {
                    let (block_row, block_column) = block_index.div_rem(&base);

                    let DynamicPosition {
                        row: base_row,
                        column: base_column,
                    } = DynamicPosition {
                        row: block_row,
                        column: block_column,
                    } * base;

                    u16::from(base_row) * u16::from(side_length) + u16::from(base_column)
                })
                .collect()
        }

        assert_eq!(
            block_index_to_top_left_cell_index::BASE_2,
            generate_block_to_top_left_cell_index(2).as_slice()
        );
        assert_eq!(
            block_index_to_top_left_cell_index::BASE_3,
            generate_block_to_top_left_cell_index(3).as_slice()
        );
        assert_eq!(
            block_index_to_top_left_cell_index::BASE_4,
            generate_block_to_top_left_cell_index(4).as_slice()
        );
        assert_eq!(
            block_index_to_top_left_cell_index::BASE_5,
            generate_block_to_top_left_cell_index(5).as_slice()
        );
    }

    #[test]
    fn test_fn_block_to_top_left_pos() {
        let base2: Vec<_> = Coordinate::<Base2>::all()
            .map(SudokuBase::block_to_top_left_pos)
            .map(Position::cell_index)
            .collect();
        assert_eq!(block_index_to_top_left_cell_index::BASE_2, base2.as_slice());

        let base3: Vec<_> = Coordinate::<Base3>::all()
            .map(SudokuBase::block_to_top_left_pos)
            .map(Position::cell_index)
            .collect();
        assert_eq!(block_index_to_top_left_cell_index::BASE_3, base3.as_slice());

        let base4: Vec<_> = Coordinate::<Base4>::all()
            .map(SudokuBase::block_to_top_left_pos)
            .map(Position::cell_index)
            .collect();
        assert_eq!(block_index_to_top_left_cell_index::BASE_4, base4.as_slice());

        let base5: Vec<_> = Coordinate::<Base5>::all()
            .map(SudokuBase::block_to_top_left_pos)
            .map(Position::cell_index)
            .collect();
        assert_eq!(block_index_to_top_left_cell_index::BASE_5, base5.as_slice());
    }
}
