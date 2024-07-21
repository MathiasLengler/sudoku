use std::{
    fmt::Debug,
    iter::{FusedIterator, TrustedLen},
    marker::PhantomData,
};

use crate::base::SudokuBase;

/// An iterator the produces the exact number of items required for a group.
///
/// This enables efficient conversion to [`super::Group<Base, T>`].
///
/// # Safety
///
/// The iterator must produce exactly `Base::SIDE_LENGTH` items.
pub unsafe trait TrustedGroupSizeIter<Base: SudokuBase>:
    ExactSizeIterator + TrustedLen + FusedIterator + Iterator
{
}

/// An adapter that asserts in *debug builds* that the wrapped iterator fulfills the contract of `TrustedGroupSizeIter`.
#[derive(Debug)]
pub(crate) struct DebugAssertTrustedGroupSizeIter<
    Base: SudokuBase,
    I: ExactSizeIterator + FusedIterator + Debug,
> {
    /// # Safety
    /// Must produce exactly `Base::SIDE_LENGTH` items.
    iter: I,
    _base: PhantomData<Base>,
    #[cfg(debug_assertions)]
    count: u8,
}

impl<Base: SudokuBase, I: ExactSizeIterator + FusedIterator + Debug>
    DebugAssertTrustedGroupSizeIter<Base, I>
{
    /// Creates a new `DebugAssertTrustedGroupSizeIter`, asserting that the provided iter fullfills the contract of `TrustedGroupSizeIter`.
    ///
    /// # Safety
    ///
    /// The provided iterator must produce exactly `Base::SIDE_LENGTH` items.
    pub(crate) unsafe fn new(iter: I) -> Self {
        debug_assert!(
            iter.len() == usize::from(Base::SIDE_LENGTH),
            "Iterator reported a length of {}, expected {}",
            iter.len(),
            Base::SIDE_LENGTH
        );
        Self {
            iter,
            _base: PhantomData,
            #[cfg(debug_assertions)]
            count: 0,
        }
    }
}

impl<Base: SudokuBase, I: ExactSizeIterator + FusedIterator + Debug> Iterator
    for DebugAssertTrustedGroupSizeIter<Base, I>
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();

        #[cfg(debug_assertions)]
        {
            if next.is_some() {
                self.count += 1;
            }
            debug_assert!(
                self.count <= Base::SIDE_LENGTH,
                "Iterator {self:?} produced more than {} items",
                Base::SIDE_LENGTH
            );
            debug_assert!(
                next.is_some() || self.count == Base::SIDE_LENGTH,
                "Iterator produced less than {} items",
                Base::SIDE_LENGTH
            );
        }

        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<Base: SudokuBase, I: ExactSizeIterator + FusedIterator + Debug> ExactSizeIterator
    for DebugAssertTrustedGroupSizeIter<Base, I>
{
    fn len(&self) -> usize {
        Base::SIDE_LENGTH.into()
    }
}

unsafe impl<Base: SudokuBase, I: ExactSizeIterator + FusedIterator + Debug> TrustedLen
    for DebugAssertTrustedGroupSizeIter<Base, I>
{
}

impl<Base: SudokuBase, I: ExactSizeIterator + FusedIterator + Debug> FusedIterator
    for DebugAssertTrustedGroupSizeIter<Base, I>
{
}

// Safety: `DebugAssertTrustedGroupSizeIter` delegates to `iter: I`.
// The length of `iter` is guaranteed to be `Base::SIDE_LENGTH`,
// therefore the safety contract of `TrustedGroupSizeIter` is upheld.
unsafe impl<Base: SudokuBase, I: ExactSizeIterator + FusedIterator + Debug>
    TrustedGroupSizeIter<Base> for DebugAssertTrustedGroupSizeIter<Base, I>
{
}
