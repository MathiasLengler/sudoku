use std::fmt::Debug;

use num::{PrimInt, Unsigned};

use nonzero_ext::NonZero;

pub trait SudokuValue
where
    Self: NonZero + Copy + Clone + Eq + Ord + Debug + Send,
    Self::Primitive: PrimInt + Unsigned,
{
}

impl<T> SudokuValue for T
where
    T: NonZero + Copy + Clone + Eq + Ord + Debug + Send,
    T::Primitive: PrimInt + Unsigned,
{
}
