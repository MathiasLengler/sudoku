use std::fmt::{Binary, Debug, Display};
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign, Shl};

use num::traits::{
    CheckedShl, CheckedShr, ConstOne, ConstZero, NumAssignOps, Unsigned, WrappingAdd, WrappingMul,
    WrappingNeg, WrappingShl, WrappingShr, WrappingSub,
};
use num::PrimInt;

use consts::*;
pub(crate) use enum_impl::match_base_enum;
pub use enum_impl::BaseEnum;

use crate::error::{Error, Result};
use crate::position::Coordinate;
use crate::position::Position;
use crate::unsafe_utils::get_unchecked;

pub mod consts {
    // Aliases
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

    pub const ALL_SIDE_LENGTHS: [u8; 4] = [
        Base2::SIDE_LENGTH,
        Base3::SIDE_LENGTH,
        Base4::SIDE_LENGTH,
        Base5::SIDE_LENGTH,
    ];
}

mod private {
    use super::consts::*;

    pub trait Sealed {}

    impl Sealed for Base2 {}
    impl Sealed for Base3 {}
    impl Sealed for Base4 {}
    impl Sealed for Base5 {}
}

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
    //! # Safety
    //! Each `array` must fulfill the following properties for its respective `Base`:
    //! - For all values (`block_index`): `block_index < Base::SIDE_LENGTH`
    //! - `array.len() == Base::CELL_COUNT`
    use super::*;

    const fn assert_u16(value: usize) -> u16 {
        assert!(value <= u16::MAX as usize, "Value exceeds u16::MAX");
        #[allow(clippy::cast_possible_truncation)]
        {
            value as u16
        }
    }

    const fn assert_u8(value: u16) -> u8 {
        assert!(value <= u8::MAX as u16, "Value exceeds u8::MAX");
        #[allow(clippy::cast_possible_truncation)]
        {
            value as u8
        }
    }

    const fn const_generate_cell_index_to_block_index<const BASE: u8, const CELL_COUNT: usize>(
    ) -> [u8; CELL_COUNT] {
        assert!(
            base_to_cell_count(BASE) as usize == CELL_COUNT,
            "Invalid CELL_COUNT for BASE"
        );
        let cell_count = assert_u16(CELL_COUNT);
        let base_u16 = BASE as u16;

        let mut out = [0u8; CELL_COUNT];

        let mut i: u16 = 0;
        while i < cell_count {
            let starting_block_index = assert_u8(base_u16 * (i / (base_u16 * base_u16 * base_u16)));
            let block_row_offset = assert_u8((i / base_u16) % base_u16);
            out[i as usize] = starting_block_index + block_row_offset;

            i += 1;
        }

        out
    }

    pub(super) static BASE_2: &[u8; base_to_cell_count(2) as usize] =
        &const_generate_cell_index_to_block_index::<2, { base_to_cell_count(2) as usize }>();
    pub(super) static BASE_3: &[u8; base_to_cell_count(3) as usize] =
        &const_generate_cell_index_to_block_index::<3, { base_to_cell_count(3) as usize }>();
    pub(super) static BASE_4: &[u8; base_to_cell_count(4) as usize] =
        &const_generate_cell_index_to_block_index::<4, { base_to_cell_count(4) as usize }>();
    pub(super) static BASE_5: &[u8; base_to_cell_count(5) as usize] =
        &const_generate_cell_index_to_block_index::<5, { base_to_cell_count(5) as usize }>();

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_assert_u8() {
            assert_eq!(assert_u8(u16::from(u8::MAX)), u8::MAX);
        }

        #[test]
        #[should_panic(expected = "Value exceeds u8::MAX")]
        fn test_assert_u8_panic() {
            assert_u8(u16::from(u8::MAX) + 1);
        }

        #[test]
        fn test_assert_u16() {
            assert_eq!(assert_u16(usize::from(u16::MAX)), u16::MAX);
        }

        #[test]
        #[should_panic(expected = "Value exceeds u16::MAX")]
        fn test_assert_u16_panic() {
            assert_u16(usize::from(u16::MAX) + 1);
        }
    }
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
/// This trait is sealed.
///
/// # Safety
/// This crate makes assumptions about the correct implementation of this trait.
/// An incorrect implementation could result in undefined behavior.
pub unsafe trait SudokuBase
where
    Self: Ord + Hash + Clone + Copy + Debug + Default + Send + Sync + 'static + private::Sealed,
{
    /// A variant of the enum `BaseEnum`
    ///
    /// Used for matching of bases at runtime.
    ///
    /// # Safety
    /// - `Base::DYNAMIC_BASE.into_u8() == Base::BASE`
    const ENUM: BaseEnum;

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
    /// The side length of the complete sudoku.
    /// Equals this number of cells in a group, e.g. row, column or block.
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
    ///
    /// Defines how many chars are representing a single cell in this grid format.
    const BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS: usize;

    const MINIMUM_CLUE_COUNT_FOR_UNIQUE_SOLUTION: u16;

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
        + Send
        + Sync
        // Generic bit twiddling
        + PrimInt
        + NumAssignOps
        + CheckedShl
        + CheckedShr
        + Unsigned
        + WrappingAdd
        + WrappingMul
        + WrappingShl
        + WrappingShr
        + WrappingSub
        + WrappingNeg
        + ConstOne
        + ConstZero
        + BitXorAssign
        + BitOrAssign
        + BitAndAssign
        + Shl<u8, Output = Self::CandidatesIntegral>
        // Conversions
        + Into<u32>
        + TryFrom<u32, Error: Into<Error> + Debug>;

    /// A generic array of `SIDE_LENGTH` elements, e.g. `[T; Self::SIDE_LENGTH]`.
    ///
    /// This is a workaround for the compiler error:
    /// > constant expression depends on a generic parameter
    ///
    /// # Safety
    ///
    /// The length of the array must equal `Base::SIDE_LENGTH`.
    type Group<T>: AsRef<[T]>
        + AsMut<[T]>
        + Send
        + Sync
        + Clone
        + Debug
        + IntoIterator<
            Item = T,
            IntoIter: ExactSizeIterator<Item = T> + DoubleEndedIterator<Item = T> + Clone,
        > + TryFrom<Vec<T>, Error = Vec<T>>
    where
        T: Send + Sync + Copy + Clone + Debug;

    fn group_default<T: Send + Sync + Copy + Clone + Debug + Default>() -> Self::Group<T>;
    fn group_uninit<T: Send + Sync + Copy + Clone + Debug>() -> Self::Group<MaybeUninit<T>>;
    fn group_map<T: Send + Sync + Copy + Clone + Debug, U: Send + Sync + Copy + Clone + Debug>(
        group: Self::Group<T>,
        f: impl FnMut(T) -> U,
    ) -> Self::Group<U>;
}

macro_rules! impl_sudoku_base {
    ($($type_num:ty,$base_u8:expr,$type_integral:ty,$CELL_INDEX_TO_BLOCK_INDEX:expr,$BLOCK_INDEX_TO_TOP_LEFT_CELL_INDEX:expr;)+) => {
        $(
// Safety: this private macro is only instantiated below and the correctness of the generated impls is tested.
unsafe impl SudokuBase for $type_num {
    const ENUM: BaseEnum = BaseEnum::assert_from_base_u8($base_u8);
    const BASE: u8 = $base_u8;
    const SIDE_LENGTH: u8 = base_to_side_length(Self::BASE);
    const MAX_VALUE: u8 = base_to_max_value(Self::BASE);
    const CELL_COUNT: u16 = base_to_cell_count(Self::BASE);
    const BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS: usize = Self::ENUM.binary_fixed_candidates_line_cell_chars();
    const MINIMUM_CLUE_COUNT_FOR_UNIQUE_SOLUTION: u16 = Self::ENUM.minimum_clue_count_for_unique_solution();

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

    type Group<T: Send + Sync + Copy + Clone + Debug> = [T; Self::SIDE_LENGTH as usize];

    fn group_default<T: Send + Sync + Copy + Clone + Debug + Default>() -> Self::Group<T> {
        [Default::default(); Self::SIDE_LENGTH as usize]
    }
    fn group_uninit<T: Send + Sync + Copy + Clone + Debug>() -> Self::Group<MaybeUninit<T>> {
        [const { MaybeUninit::uninit() }; Self::SIDE_LENGTH as usize]
    }
    fn group_map<T: Send + Sync + Copy + Clone + Debug, U: Send + Sync + Copy + Clone + Debug>(
        group: Self::Group<T>,
        f: impl FnMut(T) -> U,
    ) -> Self::Group<U> {
        group.map(f)
    }
}
        )+
    };
}

// Implement `SudokuBase` for all base structs
impl_sudoku_base!(
    Base2, 2, u8, cell_index_to_block_index::BASE_2, block_index_to_top_left_cell_index::BASE_2;
    Base3, 3, u16, cell_index_to_block_index::BASE_3, block_index_to_top_left_cell_index::BASE_3;
    Base4, 4, u16, cell_index_to_block_index::BASE_4, block_index_to_top_left_cell_index::BASE_4;
    Base5, 5, u32, cell_index_to_block_index::BASE_5, block_index_to_top_left_cell_index::BASE_5;
);

mod enum_impl {
    use super::*;
    use anyhow::{bail, format_err};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
    #[repr(u8)]
    pub enum BaseEnum {
        Base2 = 2,
        Base3 = 3,
        Base4 = 4,
        Base5 = 5,
    }

    /// const conversions between `u8` and `DynamicBase`
    impl BaseEnum {
        pub const fn assert_from_base_u8(base: u8) -> Self {
            assert!(2 <= base && base <= 5);

            match base {
                2 => BaseEnum::Base2,
                3 => BaseEnum::Base3,
                4 => BaseEnum::Base4,
                5 => BaseEnum::Base5,
                _unexpected_base => panic!("Unexpected base"),
            }
        }

        pub const fn into_u8(self) -> u8 {
            match self {
                BaseEnum::Base2 => 2,
                BaseEnum::Base3 => 3,
                BaseEnum::Base4 => 4,
                BaseEnum::Base5 => 5,
            }
        }
    }

    /// const definitions
    impl BaseEnum {
        pub const fn binary_fixed_candidates_line_cell_chars(self) -> usize {
            match self {
                BaseEnum::Base2 => 1,
                BaseEnum::Base3 => 2,
                BaseEnum::Base4 => 4,
                BaseEnum::Base5 => 6,
            }
        }

        pub const fn minimum_clue_count_for_unique_solution(self) -> u16 {
            match self {
                // Reference: https://math.stackexchange.com/questions/2170944/sudoku-what-is-the-relationship-between-minimum-number-of-clues-and-order-n
                BaseEnum::Base2 => 4,
                BaseEnum::Base3 => 17,
                // Unknown, guess on ~200 minimal sudokus
                BaseEnum::Base4 => 75,
                // Unknown, conservative estimate
                BaseEnum::Base5 => 76,
            }
        }
    }

    /// runtime conversion from base as `u8`
    impl TryFrom<u8> for BaseEnum {
        type Error = Error;

        fn try_from(base: u8) -> Result<Self> {
            Ok(match base {
                2 => BaseEnum::Base2,
                3 => BaseEnum::Base3,
                4 => BaseEnum::Base4,
                5 => BaseEnum::Base5,
                unexpected_base => bail!("Unexpected runtime base: {unexpected_base}"),
            })
        }
    }

    // interop with `SudokuBase`
    impl From<Base2> for BaseEnum {
        fn from(_base: Base2) -> Self {
            BaseEnum::Base2
        }
    }
    impl From<Base3> for BaseEnum {
        fn from(_base: Base3) -> Self {
            BaseEnum::Base3
        }
    }
    impl From<Base4> for BaseEnum {
        fn from(_base: Base4) -> Self {
            BaseEnum::Base4
        }
    }
    impl From<Base5> for BaseEnum {
        fn from(_base: Base5) -> Self {
            BaseEnum::Base5
        }
    }
    impl BaseEnum {
        pub fn is<Base: SudokuBase>(self) -> bool {
            self == Base::ENUM
        }
    }

    /// conversions between runtime sizes parameters and `DynamicBase`
    impl BaseEnum {
        pub fn try_from_cell_count_usize(cell_count: usize) -> Result<BaseEnum> {
            Ok(
                match u16::try_from(cell_count)
                    .map_err(|_| format_err!("Cell count {cell_count} too large"))?
                {
                    Base2::CELL_COUNT => BaseEnum::Base2,
                    Base3::CELL_COUNT => BaseEnum::Base3,
                    Base4::CELL_COUNT => BaseEnum::Base4,
                    Base5::CELL_COUNT => BaseEnum::Base5,
                    _ => bail!("Cell count {cell_count} has no valid sudoku base"),
                },
            )
        }
    }

    impl BaseEnum {
        pub fn all() -> impl Iterator<Item = Self> {
            [
                BaseEnum::Base2,
                BaseEnum::Base3,
                BaseEnum::Base4,
                BaseEnum::Base5,
            ]
            .into_iter()
        }
    }

    macro_rules! match_base_enum {
        ($base_enum_value:expr, $using_base:expr) => {{
            use $crate::base::consts::*;
            match $base_enum_value {
                BaseEnum::Base2 => {
                    type Base = Base2;
                    $using_base
                }
                BaseEnum::Base3 => {
                    type Base = Base3;
                    $using_base
                }
                BaseEnum::Base4 => {
                    type Base = Base4;
                    $using_base
                }
                BaseEnum::Base5 => {
                    type Base = Base5;
                    $using_base
                }
            }
        }};
    }

    pub(crate) use match_base_enum;

    #[cfg(feature = "wasm")]
    mod wasm {
        use itertools::Itertools;

        use super::*;

        impl ::ts_rs::TS for BaseEnum {
            type WithoutGenerics = Self;

            fn name() -> String {
                "BaseEnum".to_owned()
            }
            fn decl_concrete() -> String {
                format!("type {} = {};", Self::name(), Self::inline())
            }
            fn decl() -> String {
                let inline = Self::inline();
                format!("type {} = {inline};", Self::name())
            }
            fn inline() -> String {
                BaseEnum::all().map(Self::into_u8).join(" | ")
            }
            fn inline_flattened() -> String {
                panic!("{} cannot be flattened", Self::name())
            }
            fn output_path() -> Option<&'static std::path::Path> {
                Some(std::path::Path::new("BaseEnum.ts"))
            }
        }

        #[cfg(test)]
        #[test]
        fn export_bindings_baseenum() {
            <BaseEnum as ::ts_rs::TS>::export_all().expect("could not export type");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_assert_from_base_u8() {
            assert_eq!(BaseEnum::assert_from_base_u8(2), BaseEnum::Base2);
            assert_eq!(BaseEnum::assert_from_base_u8(3), BaseEnum::Base3);
            assert_eq!(BaseEnum::assert_from_base_u8(4), BaseEnum::Base4);
            assert_eq!(BaseEnum::assert_from_base_u8(5), BaseEnum::Base5);
        }

        #[test]
        fn test_into_u8() {
            assert_eq!(BaseEnum::Base2.into_u8(), 2);
            assert_eq!(BaseEnum::Base3.into_u8(), 3);
            assert_eq!(BaseEnum::Base4.into_u8(), 4);
            assert_eq!(BaseEnum::Base5.into_u8(), 5);
        }

        #[test]
        fn test_try_from_u8() {
            assert_eq!(BaseEnum::try_from(2).unwrap(), BaseEnum::Base2);
            assert_eq!(BaseEnum::try_from(3).unwrap(), BaseEnum::Base3);
            assert_eq!(BaseEnum::try_from(4).unwrap(), BaseEnum::Base4);
            assert_eq!(BaseEnum::try_from(5).unwrap(), BaseEnum::Base5);
        }

        #[test]
        fn test_from_base_structs() {
            assert_eq!(BaseEnum::from(Base2), BaseEnum::Base2);
            assert_eq!(BaseEnum::from(Base3), BaseEnum::Base3);
            assert_eq!(BaseEnum::from(Base4), BaseEnum::Base4);
            assert_eq!(BaseEnum::from(Base5), BaseEnum::Base5);
        }

        #[test]
        fn test_try_from_cell_count_usize() -> Result<()> {
            let test_cases = vec![
                (16, BaseEnum::Base2),
                (81, BaseEnum::Base3),
                (256, BaseEnum::Base4),
                (625, BaseEnum::Base5),
            ];

            for &(cell_count, expected_base) in &test_cases {
                let base = BaseEnum::try_from_cell_count_usize(cell_count)?;

                assert_eq!(base, expected_base);
            }

            let legal_cell_counts: Vec<_> = test_cases
                .into_iter()
                .map(|(cell_count, _)| cell_count)
                .collect();

            for cell_count in (0..=1000).filter(|x| !legal_cell_counts.contains(x)) {
                let res_base = BaseEnum::try_from_cell_count_usize(cell_count);
                assert!(
                    res_base.is_err(),
                    "Expected err, got {res_base:?} for cell_count: {cell_count}"
                );
            }
            Ok(())
        }
    }
}

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

        // Safety invariant of Base::DYNAMIC_BASE
        assert_eq!(Base::ENUM.into_u8(), Base::BASE);

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
        // Safety invariant of Base::Group<T>
        let mut group = Base::group_default::<()>();
        assert_eq!(group.as_ref().len(), usize::from(Base::SIDE_LENGTH));
        assert_eq!(group.as_mut().len(), usize::from(Base::SIDE_LENGTH));
    }

    #[test]
    fn test_base_2() {
        type Base = Base2;

        assert_eq!(Base::ENUM, BaseEnum::Base2);
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

        assert_eq!(Base::ENUM, BaseEnum::Base3);
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

        assert_eq!(Base::ENUM, BaseEnum::Base4);
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

        assert_eq!(Base::ENUM, BaseEnum::Base5);
        assert_eq!(Base::BASE, 5);
        assert_eq!(Base::SIDE_LENGTH, 25);
        assert_eq!(Base::MAX_VALUE, 25);
        assert_eq!(Base::CELL_COUNT, 625);
        assert_eq!(Base::BINARY_FIXED_CANDIDATES_LINE_CELL_CHARS, 6);

        assert_type_equals::<<Base as SudokuBase>::CandidatesIntegral, u32>();

        assert_base_invariants::<Base>();
    }

    #[test]
    fn test_cell_index_to_block_index_generator() {
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
    fn test_cell_index_to_block_index_invariants() {
        fn assert_invariants<Base: SudokuBase>(cell_index_to_block_index: &[u8]) {
            assert_eq!(
                cell_index_to_block_index.len(),
                usize::from(Base::CELL_COUNT)
            );
            for &block_index in cell_index_to_block_index {
                assert!(block_index < Base::SIDE_LENGTH);
            }
        }

        assert_invariants::<Base2>(cell_index_to_block_index::BASE_2);
        assert_invariants::<Base3>(cell_index_to_block_index::BASE_3);
        assert_invariants::<Base4>(cell_index_to_block_index::BASE_4);
        assert_invariants::<Base5>(cell_index_to_block_index::BASE_5);
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
    fn test_block_index_to_top_left_cell_index() {
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
