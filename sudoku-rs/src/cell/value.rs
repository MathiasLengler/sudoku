use std::convert::TryFrom;
use std::fmt::Debug;
use std::num::NonZeroUsize;

use num::{PrimInt, Unsigned};

pub trait SudokuValue
where
    Self: TryFrom<usize> + PrimInt + Unsigned + Default + Copy + Clone + Debug + Send,
    Self::Error: std::error::Error,
{
    fn is_some(&self) -> bool {
        !self.is_none()
    }
    fn is_none(&self) -> bool {
        *self == Self::zero()
    }

    //noinspection RsUnresolvedReference
    fn as_usize(&self) -> usize {
        // Unwrap should be save?
        self.to_usize().unwrap()
    }
    fn as_opt_non_zero_usize(&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.as_usize())
    }
    fn as_opt_usize(&self) -> Option<usize> {
        self.as_opt_non_zero_usize()
            .map(|non_zero| non_zero.get() as usize)
    }

    fn grid_string(&self) -> String {
        if self.is_none() {
            "_".to_string()
        } else {
            self.as_usize().to_string()
        }
    }
}

impl<T> SudokuValue for T
where
    T: TryFrom<usize> + PrimInt + Unsigned + Default + Copy + Clone + Debug + Send,
    T::Error: std::error::Error,
{
}
