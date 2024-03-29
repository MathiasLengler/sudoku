use std::ops::Index;

// Thin wrapper around `[T]::get_unchecked` performing a debug assert of the index.
// # Safety
// Has the same safety requirements as `[T]::get_unchecked`.
pub(crate) unsafe fn get_unchecked<T>(slice: &[T], index: usize) -> &T {
    debug_assert!({
        // Panics if the index is out of bounds.
        slice.index(index);
        true
    });

    // Safety: must be upheld by the caller.
    unsafe { slice.get_unchecked(index) }
}

// Thin wrapper around `[T]::get_unchecked_mut` performing a debug assert of the index.
// # Safety
// Has the same safety requirements as `[T]::get_unchecked_mut`.
pub(crate) unsafe fn get_unchecked_mut<T>(slice: &mut [T], index: usize) -> &mut T {
    debug_assert!({
        // Panics if the index is out of bounds.
        slice.index(index);
        true
    });

    // Safety: must be upheld by the caller.
    unsafe { slice.get_unchecked_mut(index) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_get_unchecked_panic() {
        let arr = [1, 2, 3];

        // Safety: this is UB in release mode, we are testing the checking for debug builds.
        unsafe {
            get_unchecked(&arr, 3);
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_get_unchecked_mut_panic() {
        let mut arr = [1, 2, 3];

        // Safety: this is UB in release mode, we are testing the checking for debug builds.
        unsafe {
            get_unchecked_mut(&mut arr, 3);
        }
    }
}
