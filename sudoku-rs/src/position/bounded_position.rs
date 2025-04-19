use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::ensure;
use serde::{Serialize, Serializer};

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::position::Coordinate;
use crate::position::DynamicPosition;

// TODO: use for all non-public APIs
/// The position of a cell in a grid of known size.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Position<Base: SudokuBase> {
    /// # Safety invariants
    /// - `cell_index < Base::CELL_COUNT`
    cell_index: u16,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase> Display for Position<Base> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dynamic_pos: DynamicPosition = (*self).into();
        write!(f, "{dynamic_pos}")
    }
}

/// Constructors
impl<Base: SudokuBase> Position<Base> {
    pub fn new(cell_index: u16) -> Result<Self> {
        Self::validate_cell_index(cell_index)?;
        // Safety: we have validated `cell_index` above.
        let this = unsafe { Self::new_unchecked(cell_index) };
        Ok(this)
    }

    /// # Safety
    ///
    /// `cell_index < Base::CELL_COUNT` must be true.
    pub(crate) unsafe fn new_unchecked(cell_index: u16) -> Self {
        let this = Self {
            cell_index,
            _base: PhantomData,
        };
        this.debug_assert();
        this
    }

    pub fn with_block_and_row_major_index(
        (block, row_major_index): (Coordinate<Base>, Coordinate<Base>),
    ) -> Self {
        let block_top_left = Base::block_to_top_left_pos(block);
        let (block_top_left_row, block_top_left_column) = block_top_left.to_row_and_column();
        let (block_row, block_column) = row_major_index.to_block_row_and_column();
        // Safety: the top-left cell in a block has a `Base::BASE -1` cells to the left and bottom of it.
        // Therefore, the indexes remain in-bounds.
        unsafe {
            (
                Coordinate::new_unchecked(block_top_left_row.get() + block_row.get()),
                Coordinate::new_unchecked(block_top_left_column.get() + block_column.get()),
            )
        }
        .into()
    }

    pub fn with_block_and_column_major_index(
        (block, column_major_index): (Coordinate<Base>, Coordinate<Base>),
    ) -> Self {
        let block_top_left = Base::block_to_top_left_pos(block);
        let (block_top_left_row, block_top_left_column) = block_top_left.to_row_and_column();
        let (block_column, block_row) = column_major_index.to_block_row_and_column();
        // Safety: the top-left cell in a block has a `Base::BASE -1` cells to the left and bottom of it.
        // Therefore, the indexes remain in-bounds.
        unsafe {
            (
                Coordinate::new_unchecked(block_top_left_row.get() + block_row.get()),
                Coordinate::new_unchecked(block_top_left_column.get() + block_column.get()),
            )
        }
        .into()
    }

    // TODO: other corners
    pub fn top_left() -> Self {
        Position::default()
    }
}

impl<Base: SudokuBase> From<(Coordinate<Base>, Coordinate<Base>)> for Position<Base> {
    fn from((row, column): (Coordinate<Base>, Coordinate<Base>)) -> Self {
        row.debug_assert();
        column.debug_assert();

        let cell_index =
            u16::from(row.get()) * u16::from(Base::SIDE_LENGTH) + u16::from(column.get());

        // Safety: the calculation for `cell_index` always remains in-bounds,
        // since `row` and `column` are each bounds checked at creation-time.
        unsafe { Self::new_unchecked(cell_index) }
    }
}

impl<Base: SudokuBase> TryFrom<(u8, u8)> for Position<Base> {
    type Error = Error;

    fn try_from((row, column): (u8, u8)) -> Result<Self> {
        let row = Coordinate::<Base>::try_from(row)?;
        let column = Coordinate::<Base>::try_from(column)?;
        Ok((row, column).into())
    }
}

impl<Base: SudokuBase> TryFrom<u16> for Position<Base> {
    type Error = Error;

    fn try_from(cell_index: u16) -> Result<Self> {
        Self::new(cell_index)
    }
}

impl<Base: SudokuBase> TryFrom<DynamicPosition> for Position<Base> {
    type Error = Error;

    fn try_from(DynamicPosition { row, column }: DynamicPosition) -> Result<Self> {
        (row, column).try_into()
    }
}

/// Validation
impl<Base: SudokuBase> Position<Base> {
    fn validate_cell_index(cell_index: u16) -> Result<()> {
        ensure!(cell_index < Base::CELL_COUNT);
        Ok(())
    }

    fn validate(self) -> Result<()> {
        Self::validate_cell_index(self.cell_index)
    }

    fn assert(self) {
        self.validate().unwrap();
    }

    pub(crate) fn debug_assert(self) {
        debug_assert!({
            self.assert();
            true
        });
    }
}

/// Getters
impl<Base: SudokuBase> Position<Base> {
    /// Get `cell_index` as a `u16`.
    /// Guaranteed to satisfy `cell_index < Base::CELL_COUNT`
    pub fn cell_index(self) -> u16 {
        self.cell_index
    }

    pub fn to_row(self) -> Coordinate<Base> {
        let row = self.cell_index / u16::from(Base::SIDE_LENGTH);

        // Safety: the calculation for `row` always remains in-bounds.
        unsafe { Coordinate::new_unchecked_u16(row) }
    }

    pub fn to_column(self) -> Coordinate<Base> {
        let column = self.cell_index % u16::from(Base::SIDE_LENGTH);

        // Safety: the calculation for `column` always remains in-bounds.
        unsafe { Coordinate::new_unchecked_u16(column) }
    }

    pub fn to_block(self) -> Coordinate<Base> {
        Base::pos_to_block(self)
    }

    pub fn to_row_and_column(self) -> (Coordinate<Base>, Coordinate<Base>) {
        (self.to_row(), self.to_column())
    }

    /// Get the block coordinate of block containing this position,
    /// as well as the index of the cell inside the block as a row- and column-major block coordinate.
    ///
    /// Example for the top left block in base 2:
    /// ```text
    /// (0,0,0) | (0,1,2)
    /// (0,2,1) | (0,3,3)
    /// ```
    pub fn to_block_and_indexes(self) -> (Coordinate<Base>, Coordinate<Base>, Coordinate<Base>) {
        let block = self.to_block();
        let (row, column) = self.to_row_and_column();
        let block_top_left_pos = Base::block_to_top_left_pos(block);
        let (block_top_left_row, block_top_left_column) = block_top_left_pos.to_row_and_column();

        let row_offset = row.get() - block_top_left_row.get();
        let column_offset = column.get() - block_top_left_column.get();

        let row_major_block_index =
            // Safety: a block contains a maximum of `Base::SIDE_LENGTH` cells. 
            unsafe { Coordinate::new_unchecked(row_offset * Base::BASE + column_offset) };
        let column_major_block_index =
            // Safety: a block contains a maximum of `Base::SIDE_LENGTH` cells. 
            unsafe { Coordinate::new_unchecked(column_offset * Base::BASE + row_offset) };

        (block, row_major_block_index, column_major_block_index)
    }
}

// TODO: optimize
/// Iterators
impl<Base: SudokuBase> Position<Base> {
    /// All grid positions in row-major order.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..Base::CELL_COUNT).map(|cell_index|
            // Safety: `cell_index` remains in-bounds
            unsafe { Self::new_unchecked(cell_index) })
    }

    /// All grid positions in column-major order.
    pub fn all_column_major() -> impl Iterator<Item = Self> {
        Self::all_columns().flatten()
    }

    pub fn row(row: Coordinate<Base>) -> impl Iterator<Item = Self> {
        let first_cell_index = row.get_u16() * u16::from(Base::SIDE_LENGTH);
        (first_cell_index..first_cell_index + u16::from(Base::SIDE_LENGTH)).map(|cell_index|
            // Safety: `cell_index` remains in-bounds
            unsafe { Self::new_unchecked(cell_index) })
    }

    pub fn all_rows() -> impl Iterator<Item = impl Iterator<Item = Self>> {
        Coordinate::all().map(Self::row)
    }

    pub fn column(column: Coordinate<Base>) -> impl Iterator<Item = Self> {
        let first_cell_index = column.get_u16();

        (first_cell_index..Base::CELL_COUNT)
            .step_by(usize::from(Base::SIDE_LENGTH))
            .map(|cell_index|
            // Safety: `cell_index` remains in-bounds
            unsafe { Self::new_unchecked(cell_index) })
    }

    pub fn all_columns() -> impl Iterator<Item = impl Iterator<Item = Self>> {
        Coordinate::all().map(Self::column)
    }

    /// Iterator over all `Position`s of a block.
    ///
    /// The block positions are yielded in row-major order.
    pub fn block(block: Coordinate<Base>) -> impl Iterator<Item = Self> {
        let block_top_left = Base::block_to_top_left_pos(block);

        (block_top_left.cell_index()..)
            .step_by(usize::from(Base::SIDE_LENGTH))
            .take(usize::from(Base::BASE))
            .flat_map(|block_cell_index_left| {
                (block_cell_index_left..(block_cell_index_left + u16::from(Base::BASE))).map(
                    |cell_index|
                        // Safety: `cell_index` remains in-bounds
                        unsafe { Position::new_unchecked(cell_index) },
                )
            })
    }

    /// Iterator over all `Position`s of a block.
    ///
    /// The block positions are yielded in column-major order.
    pub fn block_column_major(block: Coordinate<Base>) -> impl Iterator<Item = Self> {
        let base_usize = usize::from(Base::BASE);

        let block_top_left = Base::block_to_top_left_pos(block);

        (block_top_left.cell_index()..)
            .take(base_usize)
            .flat_map(move |block_cell_index_top| {
                (block_cell_index_top..)
                    .step_by(usize::from(Base::SIDE_LENGTH))
                    .take(base_usize)
                    .map(|cell_index|
                        // Safety: `cell_index` remains in-bounds
                        unsafe { Position::new_unchecked(cell_index) })
            })
    }

    /// Nested iterator over all block positions.
    ///
    /// The blocks *and* positions are visited in row-major order.
    pub fn all_blocks() -> impl Iterator<Item = impl Iterator<Item = Self>> {
        Coordinate::all().map(Self::block)
    }

    /// Iterator over all positions which are in the top left of a block.
    ///
    /// The blocks are visited in a row-major order.
    pub fn all_blocks_top_left() -> impl Iterator<Item = Self> {
        Coordinate::all().map(Base::block_to_top_left_pos)
    }

    // TODO: optimize
    //  collect into `[Position: SIDE_LENGTH]`?
    pub fn all_groups() -> impl Iterator<Item = impl Iterator<Item = Self> + Clone> {
        Self::all_rows()
            .map(|rows| rows.collect::<Vec<_>>().into_iter())
            .chain(Self::all_columns().map(|columns| columns.collect::<Vec<_>>().into_iter()))
            .chain(Self::all_blocks().map(|blocks| blocks.collect::<Vec<_>>().into_iter()))
    }
}

// TODO: data structures for
//  - Set of Positions
//   - Current representation: `Vec<Position<Base>>`
//    - uniqueness of Positions is not ensured
//    - Redundant order information
//   - Options:
//    - wrapper around std HashSet/BTreeSet
//    - re-use Base::CandidatesGroup as bitset? correct bit count, but split into SIDE_LENGTH candidates
//  - Ordered Set of Positions
//   - Current representation: `Vec<Position<Base>>`
//    - uniqueness of Positions is not ensured
//   - Options
//    - No standard data structure?

/// Set of positions
impl<Base: SudokuBase> Position<Base> {
    /// All positions *not* included in `positions`.
    pub fn complement(mut positions: Vec<Self>) -> impl Iterator<Item = Self> {
        let sorted_positions = {
            positions.sort_unstable();
            positions
        };
        Self::all()
            // Remove positions contained in sorted_positions
            .filter(move |pos| sorted_positions.binary_search(pos).is_err())
    }
}

impl<Base: SudokuBase> Serialize for Position<Base> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.cell_index())
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::base::consts::Base2;

    use super::*;

    mod constructors {
        use super::*;

        #[test]
        fn test_new() {
            assert_eq!(Position::<Base2>::new(0).unwrap().cell_index, 0);
            assert_eq!(Position::<Base2>::new(15).unwrap().cell_index, 15);
            assert!(Position::<Base2>::new(16).is_err());
        }
    }

    mod getters {
        use super::*;

        #[test]
        fn test_to_block_and_indexes() {
            Position::<Base2>::all_blocks()
                .zip_eq(vec![
                    vec![(0, 0, 0), (0, 1, 2), (0, 2, 1), (0, 3, 3)],
                    vec![(1, 0, 0), (1, 1, 2), (1, 2, 1), (1, 3, 3)],
                    vec![(2, 0, 0), (2, 1, 2), (2, 2, 1), (2, 3, 3)],
                    vec![(3, 0, 0), (3, 1, 2), (3, 2, 1), (3, 3, 3)],
                ])
                .for_each(|(block_positions, expected_block_and_indexes)| {
                    itertools::assert_equal(
                        block_positions.map(|block_pos| block_pos.to_block_and_indexes()),
                        expected_block_and_indexes.into_iter().map(
                            |(block, row_major_block_index, column_major_block_index)| {
                                (
                                    Coordinate::new(block).unwrap(),
                                    Coordinate::new(row_major_block_index).unwrap(),
                                    Coordinate::new(column_major_block_index).unwrap(),
                                )
                            },
                        ),
                    );
                });
        }
    }

    mod iterators {
        use crate::base::consts::Base5;
        use crate::position::test_utils::{consume_iter, consume_nested_iter};

        use super::*;

        #[test]
        fn test_all() {
            itertools::assert_equal(
                Position::<Base2>::all(),
                vec![
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (0, 3),
                    (1, 0),
                    (1, 1),
                    (1, 2),
                    (1, 3),
                    (2, 0),
                    (2, 1),
                    (2, 2),
                    (2, 3),
                    (3, 0),
                    (3, 1),
                    (3, 2),
                    (3, 3),
                ]
                .into_iter()
                .map(|pos| pos.try_into().unwrap()),
            );
        }

        #[test]
        fn test_all_column_major() {
            itertools::assert_equal(
                Position::<Base2>::all_column_major(),
                vec![
                    (0, 0),
                    (1, 0),
                    (2, 0),
                    (3, 0),
                    (0, 1),
                    (1, 1),
                    (2, 1),
                    (3, 1),
                    (0, 2),
                    (1, 2),
                    (2, 2),
                    (3, 2),
                    (0, 3),
                    (1, 3),
                    (2, 3),
                    (3, 3),
                ]
                .into_iter()
                .map(|pos| pos.try_into().unwrap()),
            );
        }

        #[test]
        fn test_row() {
            itertools::assert_equal(
                Position::<Base2>::row(0.try_into().unwrap()),
                vec![(0, 0), (0, 1), (0, 2), (0, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::row(1.try_into().unwrap()),
                vec![(1, 0), (1, 1), (1, 2), (1, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::row(2.try_into().unwrap()),
                vec![(2, 0), (2, 1), (2, 2), (2, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::row(3.try_into().unwrap()),
                vec![(3, 0), (3, 1), (3, 2), (3, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
        }

        #[test]
        fn test_all_rows() {
            Position::<Base2>::all_rows()
                .zip_eq(vec![
                    vec![(0, 0), (0, 1), (0, 2), (0, 3)],
                    vec![(1, 0), (1, 1), (1, 2), (1, 3)],
                    vec![(2, 0), (2, 1), (2, 2), (2, 3)],
                    vec![(3, 0), (3, 1), (3, 2), (3, 3)],
                ])
                .for_each(|(actual_row, expected_row)| {
                    itertools::assert_equal(
                        actual_row,
                        expected_row.into_iter().map(|pos| pos.try_into().unwrap()),
                    );
                });
        }

        #[test]
        fn test_column() {
            itertools::assert_equal(
                Position::<Base2>::column(0.try_into().unwrap()),
                vec![(0, 0), (1, 0), (2, 0), (3, 0)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::column(1.try_into().unwrap()),
                vec![(0, 1), (1, 1), (2, 1), (3, 1)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::column(2.try_into().unwrap()),
                vec![(0, 2), (1, 2), (2, 2), (3, 2)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::column(3.try_into().unwrap()),
                vec![(0, 3), (1, 3), (2, 3), (3, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
        }

        #[test]
        fn test_all_columns() {
            Position::<Base2>::all_columns()
                .zip_eq(vec![
                    vec![(0, 0), (1, 0), (2, 0), (3, 0)],
                    vec![(0, 1), (1, 1), (2, 1), (3, 1)],
                    vec![(0, 2), (1, 2), (2, 2), (3, 2)],
                    vec![(0, 3), (1, 3), (2, 3), (3, 3)],
                ])
                .for_each(|(actual_row, expected_row)| {
                    itertools::assert_equal(
                        actual_row,
                        expected_row.into_iter().map(|pos| pos.try_into().unwrap()),
                    );
                });
        }

        #[test]
        fn test_block() {
            itertools::assert_equal(
                Position::<Base2>::block(0.try_into().unwrap()),
                vec![(0, 0), (0, 1), (1, 0), (1, 1)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::block(1.try_into().unwrap()),
                vec![(0, 2), (0, 3), (1, 2), (1, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::block(2.try_into().unwrap()),
                vec![(2, 0), (2, 1), (3, 0), (3, 1)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::block(3.try_into().unwrap()),
                vec![(2, 2), (2, 3), (3, 2), (3, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
        }

        #[test]
        fn test_all_blocks() {
            Position::<Base2>::all_blocks()
                .zip_eq(vec![
                    vec![(0, 0), (0, 1), (1, 0), (1, 1)],
                    vec![(0, 2), (0, 3), (1, 2), (1, 3)],
                    vec![(2, 0), (2, 1), (3, 0), (3, 1)],
                    vec![(2, 2), (2, 3), (3, 2), (3, 3)],
                ])
                .for_each(|(actual_row, expected_row)| {
                    itertools::assert_equal(
                        actual_row,
                        expected_row.into_iter().map(|pos| pos.try_into().unwrap()),
                    );
                });
        }

        #[test]
        fn test_block_column_major() {
            itertools::assert_equal(
                Position::<Base2>::block_column_major(0.try_into().unwrap()),
                vec![(0, 0), (1, 0), (0, 1), (1, 1)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::block_column_major(1.try_into().unwrap()),
                vec![(0, 2), (1, 2), (0, 3), (1, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::block_column_major(2.try_into().unwrap()),
                vec![(2, 0), (3, 0), (2, 1), (3, 1)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
            itertools::assert_equal(
                Position::<Base2>::block_column_major(3.try_into().unwrap()),
                vec![(2, 2), (3, 2), (2, 3), (3, 3)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
        }

        #[test]
        fn test_all_blocks_top_left() {
            itertools::assert_equal(
                Position::<Base2>::all_blocks_top_left(),
                vec![(0, 0), (0, 2), (2, 0), (2, 2)]
                    .into_iter()
                    .map(|pos| pos.try_into().unwrap()),
            );
        }

        #[test]
        fn test_iter_overflow() {
            consume_iter(Position::<Base5>::all());
            consume_iter(Position::<Base5>::row(Coordinate::max()));
            consume_nested_iter(Position::<Base5>::all_rows());
            consume_iter(Position::<Base5>::column(Coordinate::max()));
            consume_nested_iter(Position::<Base5>::all_columns());
            consume_iter(Position::<Base5>::block(Coordinate::max()));
            consume_nested_iter(Position::<Base5>::all_blocks());
        }
    }
}
