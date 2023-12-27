use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::position::{BlockCoordinate, Coordinate, Position};
use itertools::Either;
use std::fmt::{self, Display, Formatter};

/// The position of a block segment inside a sudoku grid.
///
/// A block segment is a "strip" of cells inside a block and row/column (axis).
/// It always contains `Base::BASE` cells.
///
/// # Examples
///
/// `block 0, segment 0, order: RowMajor`
/// ```text
/// ╔═══════╦═══════╗
/// ║ X │ X ║   │   ║
/// ║───┼───║───┼───║
/// ║   │   ║   │   ║
/// ╠═══════╬═══════╣
/// ║   │   ║   │   ║
/// ║───┼───║───┼───║
/// ║   │   ║   │   ║
/// ╚═══════╩═══════╝
/// ```
///
/// `block 0, segment 1, order: RowMajor`
/// ```text
/// ╔═══════╦═══════╗
/// ║   │   ║   │   ║
/// ║───┼───║───┼───║
/// ║ X │ X ║   │   ║
/// ╠═══════╬═══════╣
/// ║   │   ║   │   ║
/// ║───┼───║───┼───║
/// ║   │   ║   │   ║
/// ╚═══════╩═══════╝
/// ```
///
/// `block 1, segment 0, order: RowMajor`
/// ```text
/// ╔═══════╦═══════╗
/// ║   │   ║ X │ X ║
/// ║───┼───║───┼───║
/// ║   │   ║   │   ║
/// ╠═══════╬═══════╣
/// ║   │   ║   │   ║
/// ║───┼───║───┼───║
/// ║   │   ║   │   ║
/// ╚═══════╩═══════╝
/// ```
///
/// `block 1, segment 0, order: ColumnMajor`
/// ```text
/// ╔═══════╦═══════╗
/// ║   │   ║ X │   ║
/// ║───┼───║───┼───║
/// ║   │   ║ X │   ║
/// ╠═══════╬═══════╣
/// ║   │   ║   │   ║
/// ║───┼───║───┼───║
/// ║   │   ║   │   ║
/// ╚═══════╩═══════╝
/// ```
/// `block 1, segment 1, order: ColumnMajor`
/// ```text
/// ╔═══════╦═══════╗
/// ║   │   ║   │ X ║
/// ║───┼───║───┼───║
/// ║   │   ║   │ X ║
/// ╠═══════╬═══════╣
/// ║   │   ║   │   ║
/// ║───┼───║───┼───║
/// ║   │   ║   │   ║
/// ╚═══════╩═══════╝
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct BlockSegment<Base: SudokuBase> {
    block: Coordinate<Base>,
    segment: BlockCoordinate<Base>,
    orientation: CellOrder,
}

impl<Base: SudokuBase> Display for BlockSegment<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut segment_positions = self.segment_positions();
        let (first_pos, last_pos) = match (segment_positions.next(), segment_positions.last()) {
            (Some(first), Some(last)) => (first, last),
            (Some(first), None) => (first, first),
            (_, _) => panic!("Expected at least one segment position"),
        };

        write!(f, "{first_pos}-{last_pos}")
    }
}

impl<Base: SudokuBase> BlockSegment<Base> {
    // FIXME: change/add iteration order
    //  it could be more natural to iterate the segments in axis order.
    pub fn all() -> impl Iterator<Item = Self> {
        Self::all_with_orientation(CellOrder::RowMajor)
            .chain(Self::all_with_orientation(CellOrder::ColumnMajor))
    }

    /// Iterator over all block segments of a given orientation.
    ///
    /// The segments are visited in block-first order,
    /// e.g. all segments of block 0, then all segments of block 1, etc.
    pub fn all_with_orientation(orientation: CellOrder) -> impl Iterator<Item = Self> {
        Coordinate::<Base>::all().flat_map(move |block| {
            BlockCoordinate::<Base>::all().map(move |segment| Self {
                block,
                segment,
                orientation,
            })
        })
    }

    pub fn orientation(self) -> CellOrder {
        self.orientation
    }

    // TODO: test
    /// The index of this block segment in its containing block.
    pub fn block_segment_index(self) -> BlockCoordinate<Base> {
        self.segment
    }

    // TODO: test
    /// The index of this block segment in its containing axis.
    pub fn axis_segment_index(self) -> BlockCoordinate<Base> {
        match self.orientation {
            CellOrder::RowMajor => self.block.to_block_column(),
            CellOrder::ColumnMajor => self.block.to_block_row(),
        }
    }

    pub fn axis(self) -> Coordinate<Base> {
        let Self {
            block,
            segment,
            orientation,
        } = self;
        match orientation {
            CellOrder::RowMajor => (block.to_block_row(), segment),
            CellOrder::ColumnMajor => (block.to_block_column(), segment),
        }
        .into()
    }

    pub fn block(self) -> Coordinate<Base> {
        self.block
    }

    pub fn axis_positions(self) -> impl Iterator<Item = Position<Base>> {
        let axis = self.axis();
        match self.orientation {
            CellOrder::RowMajor => Either::Left(Position::row(axis)),
            CellOrder::ColumnMajor => Either::Right(Position::column(axis)),
        }
    }

    pub fn axis_position(self, axis_index: Coordinate<Base>) -> Position<Base> {
        match self.orientation {
            CellOrder::RowMajor => (self.axis(), axis_index),
            CellOrder::ColumnMajor => (axis_index, self.axis()),
        }
        .into()
    }

    pub fn block_positions(self) -> impl Iterator<Item = Position<Base>> {
        match self.orientation {
            CellOrder::RowMajor => Either::Left(Position::block(self.block)),
            CellOrder::ColumnMajor => Either::Right(Position::block_column_major(self.block)),
        }
    }

    pub fn block_position(self, block_index: Coordinate<Base>) -> Position<Base> {
        match self.orientation {
            CellOrder::RowMajor => Position::block(self.block)
                .nth(block_index.get_usize())
                .unwrap(),
            CellOrder::ColumnMajor => Position::block_column_major(self.block)
                .nth(block_index.get_usize())
                .unwrap(),
        }
    }

    pub fn segment_positions(self) -> impl Iterator<Item = Position<Base>> {
        // TODO: simplify/optimize
        let (block_row, block_column) = self.block.to_block_row_and_column();
        let base_usize = usize::from(Base::BASE);
        self.axis_positions()
            .skip(match self.orientation {
                CellOrder::RowMajor => block_column.get_usize() * base_usize,
                CellOrder::ColumnMajor => block_row.get_usize() * base_usize,
            })
            .take(base_usize)
    }

    pub fn axis_mask(self) -> Candidates<Base> {
        Candidates::block_segmentation_mask(self.axis_segment_index())
    }

    pub fn block_mask(self) -> Candidates<Base> {
        Candidates::block_segmentation_mask(self.block_segment_index())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub enum CellOrder {
    #[default]
    RowMajor,
    ColumnMajor,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::Base2;
    use itertools::{assert_equal, zip_eq};

    #[test]
    fn test_all() {
        assert_equal(
            BlockSegment::<Base2>::all(),
            vec![
                (0, 0, CellOrder::RowMajor),
                (0, 1, CellOrder::RowMajor),
                (1, 0, CellOrder::RowMajor),
                (1, 1, CellOrder::RowMajor),
                (2, 0, CellOrder::RowMajor),
                (2, 1, CellOrder::RowMajor),
                (3, 0, CellOrder::RowMajor),
                (3, 1, CellOrder::RowMajor),
                (0, 0, CellOrder::ColumnMajor),
                (0, 1, CellOrder::ColumnMajor),
                (1, 0, CellOrder::ColumnMajor),
                (1, 1, CellOrder::ColumnMajor),
                (2, 0, CellOrder::ColumnMajor),
                (2, 1, CellOrder::ColumnMajor),
                (3, 0, CellOrder::ColumnMajor),
                (3, 1, CellOrder::ColumnMajor),
            ]
            .into_iter()
            .map(|(block, segment, orientation)| BlockSegment {
                block: block.try_into().unwrap(),
                segment: segment.try_into().unwrap(),
                orientation,
            }),
        );
    }

    #[test]
    fn test_all_with_orientation_row_major() {
        assert_equal(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::RowMajor),
            vec![
                (0, 0),
                (0, 1),
                (1, 0),
                (1, 1),
                (2, 0),
                (2, 1),
                (3, 0),
                (3, 1),
            ]
            .into_iter()
            .map(|(block, segment)| BlockSegment {
                block: block.try_into().unwrap(),
                segment: segment.try_into().unwrap(),
                orientation: CellOrder::RowMajor,
            }),
        );
    }

    #[test]
    fn test_all_with_orientation_column_major() {
        assert_equal(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::ColumnMajor),
            vec![
                (0, 0),
                (0, 1),
                (1, 0),
                (1, 1),
                (2, 0),
                (2, 1),
                (3, 0),
                (3, 1),
            ]
            .into_iter()
            .map(|(block, segment)| BlockSegment {
                block: block.try_into().unwrap(),
                segment: segment.try_into().unwrap(),
                orientation: CellOrder::ColumnMajor,
            }),
        );
    }

    #[test]
    fn test_axis_row_major() {
        assert_equal(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::RowMajor)
                .map(|block_segment| block_segment.axis()),
            vec![0, 1, 0, 1, 2, 3, 2, 3]
                .into_iter()
                .map(|row| Coordinate::new(row).unwrap()),
        );
    }

    #[test]
    fn test_axis_column_major() {
        assert_equal(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::ColumnMajor)
                .map(|block_segment| block_segment.axis()),
            vec![0, 1, 2, 3, 0, 1, 2, 3]
                .into_iter()
                .map(|row| Coordinate::new(row).unwrap()),
        );
    }

    #[test]
    fn test_axis_positions_row_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::RowMajor)
                .map(|block_segment| block_segment.axis_positions()),
            vec![0, 1, 0, 1, 2, 3, 2, 3],
        )
        .for_each(|(positions, expected_row)| {
            assert_equal(positions, Position::row(expected_row.try_into().unwrap()));
        });
    }

    #[test]
    fn test_axis_positions_column_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::ColumnMajor)
                .map(|block_segment| block_segment.axis_positions()),
            vec![0, 1, 2, 3, 0, 1, 2, 3],
        )
        .for_each(|(positions, expected_column)| {
            assert_equal(
                positions,
                Position::column(expected_column.try_into().unwrap()),
            );
        });
    }

    #[test]
    fn test_block_positions_row_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::RowMajor)
                .map(|block_segment| block_segment.block_positions()),
            vec![0, 0, 1, 1, 2, 2, 3, 3],
        )
        .for_each(|(positions, expected_block)| {
            assert_equal(
                positions,
                Position::block(expected_block.try_into().unwrap()),
            );
        });
    }
    #[test]
    fn test_block_positions_column_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::ColumnMajor)
                .map(|block_segment| block_segment.block_positions()),
            vec![0, 0, 1, 1, 2, 2, 3, 3],
        )
        .for_each(|(positions, expected_block)| {
            assert_equal(
                positions,
                Position::block_column_major(expected_block.try_into().unwrap()),
            );
        });
    }

    #[test]
    fn test_segment_positions_row_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::RowMajor)
                .map(|block_segment| block_segment.segment_positions()),
            vec![
                vec![(0, 0), (0, 1)],
                vec![(1, 0), (1, 1)],
                vec![(0, 2), (0, 3)],
                vec![(1, 2), (1, 3)],
                vec![(2, 0), (2, 1)],
                vec![(3, 0), (3, 1)],
                vec![(2, 2), (2, 3)],
                vec![(3, 2), (3, 3)],
            ],
        )
        .for_each(|(positions, expected_positions)| {
            assert_equal(
                positions,
                expected_positions
                    .into_iter()
                    .map(|(row, column)| Position::try_from((row, column)).unwrap()),
            );
        });
    }

    #[test]
    fn test_segment_positions_column_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::ColumnMajor)
                .map(|block_segment| block_segment.segment_positions()),
            vec![
                vec![(0, 0), (1, 0)],
                vec![(0, 1), (1, 1)],
                vec![(0, 2), (1, 2)],
                vec![(0, 3), (1, 3)],
                vec![(2, 0), (3, 0)],
                vec![(2, 1), (3, 1)],
                vec![(2, 2), (3, 2)],
                vec![(2, 3), (3, 3)],
            ],
        )
        .for_each(|(positions, expected_positions)| {
            assert_equal(
                positions,
                expected_positions
                    .into_iter()
                    .map(|(row, column)| Position::try_from((row, column)).unwrap()),
            );
        });
    }

    #[test]
    fn test_axis_mask_row_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::RowMajor)
                .map(|block_segment| block_segment.axis_mask()),
            vec![
                0b0011, //
                0b0011, //
                0b1100, //
                0b1100, //
                0b0011, //
                0b0011, //
                0b1100, //
                0b1100, //
            ],
        )
        .for_each(|(candidates, expected_candidates_integral)| {
            assert_equal(
                candidates,
                Candidates::with_integral(expected_candidates_integral),
            );
        });
    }

    #[test]
    fn test_axis_mask_column_major() {
        zip_eq(
            BlockSegment::<Base2>::all_with_orientation(CellOrder::ColumnMajor)
                .map(|block_segment| block_segment.axis_mask()),
            vec![
                0b0011, //
                0b0011, //
                0b0011, //
                0b0011, //
                0b1100, //
                0b1100, //
                0b1100, //
                0b1100, //
            ],
        )
        .for_each(|(candidates, expected_candidates_integral)| {
            assert_equal(
                candidates,
                Candidates::with_integral(expected_candidates_integral),
            );
        });
    }

    #[test]
    fn test_block_mask() {
        for orientation in [CellOrder::RowMajor, CellOrder::ColumnMajor] {
            zip_eq(
                BlockSegment::<Base2>::all_with_orientation(orientation)
                    .map(|block_segment| block_segment.block_mask()),
                vec![
                    0b0011, //
                    0b1100, //
                    0b0011, //
                    0b1100, //
                    0b0011, //
                    0b1100, //
                    0b0011, //
                    0b1100, //
                ],
            )
            .for_each(|(candidates, expected_candidates_integral)| {
                assert_equal(
                    candidates,
                    Candidates::with_integral(expected_candidates_integral),
                );
            });
        }
    }
}
