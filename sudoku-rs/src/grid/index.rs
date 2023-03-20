pub mod coordinate;
pub mod position;

// Used by benchmarking harness
#[doc(hidden)]
pub mod test_utils {
    use std::hint::black_box;

    use crate::base::consts::*;
    use crate::grid::index::position::Position;

    // TODO: remove after optimization
    pub fn debug_asm() {
        consume_iter(Position::<Base3>::block(black_box(3.try_into().unwrap())));
    }

    pub fn consume_iter<T>(iter: impl Iterator<Item = T>) {
        iter.for_each(|item| {
            black_box(item);
        });
    }

    pub fn consume_nested_iter<T>(iter: impl Iterator<Item = impl Iterator<Item = T>>) {
        iter.for_each(|nested_iter| {
            nested_iter.for_each(|item| {
                black_box(item);
            });
        });
    }
}
