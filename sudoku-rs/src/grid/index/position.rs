use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;

use anyhow::ensure;
use serde::{Serialize, Serializer};

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::grid::index::coordinate::Coordinate;
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
        write!(f, "{}", dynamic_pos)
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

    fn validate(&self) -> Result<()> {
        Self::validate_cell_index(self.cell_index)
    }

    fn assert(&self) {
        self.validate().unwrap();
    }

    pub(crate) fn debug_assert(&self) {
        debug_assert!({
            self.assert();
            true
        });
    }
}

/// Getters
impl<Base: SudokuBase> Position<Base> {
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
}

// TODO: optimize
/// Iterators
impl<Base: SudokuBase> Position<Base> {
    pub fn all() -> impl Iterator<Item = Self> {
        (0..Base::CELL_COUNT).map(|cell_index|
            // Safety: `cell_index` remains in-bounds
            unsafe { Self::new_unchecked(cell_index) })
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

    pub fn all_blocks() -> impl Iterator<Item = impl Iterator<Item = Self>> {
        Coordinate::all().map(Self::block)
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

#[cfg(feature = "wasm")]
mod wasm {
    use ts_rs::TS;

    use super::*;

    impl<Base: SudokuBase> TS for Position<Base> {
        const EXPORT_TO: Option<&'static str> = Some("bindings/Position.ts");
        fn decl() -> String {
            "type Position = number;".to_owned()
        }
        fn name() -> String {
            "Position".to_owned()
        }
        fn name_with_type_args(_args: Vec<String>) -> String {
            Self::name()
        }
        fn inline() -> String {
            "number".to_owned()
        }
        fn dependencies() -> Vec<ts_rs::Dependency> {
            vec![]
        }
        fn transparent() -> bool {
            false
        }
    }

    #[cfg(test)]
    #[test]
    fn export_bindings_value() {
        use crate::base::consts::Base3;

        <Position<Base3> as ts_rs::TS>::export().expect("could not export type");
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(Position::<Base2>::new(0).unwrap().cell_index, 0);
        assert_eq!(Position::<Base2>::new(15).unwrap().cell_index, 15);
        assert!(Position::<Base2>::new(16).is_err());
    }

    mod iterators {
        use super::*;
        use crate::base::consts::Base5;
        use crate::grid::index::test_utils::{consume_iter, consume_nested_iter};

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
