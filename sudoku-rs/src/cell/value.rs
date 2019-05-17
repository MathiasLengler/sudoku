use std::fmt::Debug;
use std::hash::Hash;

use nonzero_ext::NonZero;
use num::{PrimInt, Unsigned};

pub trait SudokuValue
where
    Self: NonZero + Copy + Clone + Eq + Ord + Hash + Debug + Send,
    Self::Primitive: PrimInt + Unsigned,
{
}

impl<T> SudokuValue for T
where
    T: NonZero + Copy + Clone + Eq + Ord + Hash + Debug + Send,
    T::Primitive: PrimInt + Unsigned,
{
}
