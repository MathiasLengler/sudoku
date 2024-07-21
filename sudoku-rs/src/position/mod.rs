// TODO: introduce Positioned<Base, T>(Position<Base>, T)
//  replace current usages of (Position<Base>, ...)
//  also useful for Grid iterators
//  could simply be an alias for a tuple `(Position<Base>, T)`

// TODO: rename module to `grid_position`?

// TODO: can we reduce the boilerplate for bounded types via an external crate?

pub use bounded_block_coordinate::BlockCoordinate;
pub use bounded_block_segment::{BlockSegment, CellOrder};
pub use bounded_coordinate::Coordinate;
pub use bounded_position::{debug_asm, Position};
pub use dynamic::DynamicPosition;
pub use position_map::{Merge, PositionMap};

mod bounded_block_coordinate;
mod bounded_block_segment;
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
