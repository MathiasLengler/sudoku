use std::fmt::Debug;
use std::hash::Hash;
use std::mem::size_of;

use typenum::{
    consts::{U1, U2, U3, U4, U5},
    Unsigned,
};

pub mod consts {
    pub use typenum::consts::*;
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

// TODO: evaluate `as` casting of constants
pub trait SudokuBase
where
    Self: Ord + Hash + Clone + Copy + Debug + Default,
{
    const BASE: u8;
    const SIDE_LENGTH: u8;
    const MAX_VALUE: u8;
    const CELL_COUNT: u16;
    const CANDIDATES_CAPACITY: usize;

    type CandidatesArrayElement: bitvec::store::BitStore;
    type CandidatesArray: bitvec::view::BitViewSized + Ord + Hash + Clone + Debug + Default;
}

macro_rules! impl_sudoku_base {
    ($($type_num:ty),+) => {
        $(
impl SudokuBase for $type_num {
    const BASE: u8 = Self::U8;
    const SIDE_LENGTH: u8 = base_to_side_length(Self::BASE);
    const MAX_VALUE: u8 = base_to_max_value(Self::BASE);
    const CELL_COUNT: u16 = base_to_cell_count(Self::BASE);
    const CANDIDATES_CAPACITY: usize =
        base_to_candidates_capacity::<Self::CandidatesArrayElement>(Self::BASE);

    type CandidatesArrayElement = u8;
    type CandidatesArray = [Self::CandidatesArrayElement; Self::CANDIDATES_CAPACITY];
}
        )+
    };
}

// All sudoku bases supported by DynamicSudoku, and U1 for testing.
impl_sudoku_base!(U1, U2, U3, U4, U5);

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
