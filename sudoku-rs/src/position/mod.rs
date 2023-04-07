// TODO: introduce Positioned<Base, T>(Position<Base>, T)
//  replace current usages of (Position<Base>, ...)
//  also useful for Grid iterators

pub use bounded_coordinate::Coordinate;
pub use bounded_position::Position;
pub use dynamic::DynamicPosition;
pub use position_map::{Merge, PositionMap};

mod bounded_coordinate;
mod bounded_position;
mod dynamic;
mod position_map;

// Used by benchmarking harness
#[doc(hidden)]
pub mod test_utils {
    use std::hint::black_box;

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
