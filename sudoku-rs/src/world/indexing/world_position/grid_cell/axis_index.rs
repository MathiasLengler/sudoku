use std::num::NonZeroUsize;

use num::Integer;

use crate::{
    base::SudokuBase,
    position::Coordinate,
    world::{AxisOrdering, WorldGridPosition},
};

// TODO: test every method

/// The axis index (row or column) of a cell inside a specific grid in the world.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(in crate::world::indexing) struct WorldGridCellAxisIndex<Base: SudokuBase> {
    /// The axis index of the grid in the world.
    world_grid_axis_index: usize,
    /// The axis index of the cell inside this grid.
    cell_axis_index: Coordinate<Base>,
}

impl<Base: SudokuBase> WorldGridCellAxisIndex<Base> {
    /// For the given `world_cell_axis_index`, returns the equivalent `WorldGridCellAxisIndex`
    /// with the greatest `world_grid_axis_index` within bounds and the lowest `cell_axis_index`.
    pub(in crate::world::indexing) fn from_world_cell_axis_index(
        world_cell_axis_index: usize,
        world_grid_axis_count: NonZeroUsize,
        overlap: u8,
    ) -> Self {
        let grid_stride_usize = WorldGridPosition::stride_usize::<Base>(overlap);

        let mut this = Self {
            world_grid_axis_index: world_cell_axis_index / grid_stride_usize,
            cell_axis_index: Coordinate::new(
                (world_cell_axis_index % grid_stride_usize)
                    .try_into()
                    .unwrap(),
            )
            .unwrap(),
        };

        if this.world_grid_axis_index == world_grid_axis_count.get() {
            this = this
                .to_neighboring_grid_axis_index(AxisOrdering::Less, world_grid_axis_count, overlap)
                .unwrap();
        }

        debug_assert!(this.world_grid_axis_index < world_grid_axis_count.get());

        this
    }

    pub(in crate::world::indexing) fn world_grid_axis_index(self) -> usize {
        self.world_grid_axis_index
    }
    pub(in crate::world::indexing) fn cell_axis_index(self) -> Coordinate<Base> {
        self.cell_axis_index
    }

    pub(in crate::world::indexing) fn overlap_neighbor(
        self,
        world_grid_axis_count: NonZeroUsize,
        overlap: u8,
    ) -> Option<(Self, AxisOrdering)> {
        let grid_stride = WorldGridPosition::stride::<Base>(overlap);
        let cell_axis_index = self.cell_axis_index.get();

        let neighbor_ordering = if cell_axis_index < overlap {
            // inside start overlap => neighbor before
            Some(AxisOrdering::Less)
        } else if cell_axis_index >= grid_stride {
            // inside end overlap => neighbor after
            Some(AxisOrdering::Greater)
        } else {
            // not in any overlap
            None
        };

        neighbor_ordering.and_then(|neighbor_ordering| {
            self.to_neighboring_grid_axis_index(neighbor_ordering, world_grid_axis_count, overlap)
                .map(|neighbor| (neighbor, neighbor_ordering))
        })
    }

    fn to_neighboring_grid_axis_index(
        self,
        neighbor_ordering: AxisOrdering,
        world_grid_axis_count: NonZeroUsize,
        overlap: u8,
    ) -> Option<Self> {
        let Self {
            world_grid_axis_index,
            cell_axis_index,
        } = self;
        let grid_stride = WorldGridPosition::stride::<Base>(overlap);
        let cell_axis_index = cell_axis_index.get();

        match neighbor_ordering {
            AxisOrdering::Less => {
                if world_grid_axis_index == 0 {
                    // no neighbor before
                    None
                } else {
                    // neighbor before
                    Some(Self {
                        world_grid_axis_index: world_grid_axis_index - 1,
                        cell_axis_index: (cell_axis_index + grid_stride).try_into().unwrap(),
                    })
                }
            }
            AxisOrdering::Greater => {
                if world_grid_axis_index == world_grid_axis_count.get() - 1 {
                    // no neighbor after
                    None
                } else {
                    // neighbor after
                    Some(Self {
                        world_grid_axis_index: world_grid_axis_index + 1,
                        cell_axis_index: (cell_axis_index - grid_stride).try_into().unwrap(),
                    })
                }
            }
        }
    }

    /// Normalize `self` to the nearest grid axis index.
    ///
    /// If the index is outside of an overlap, returns `self` unchanged.
    /// If the index is inside an overlap, returns the same index but from the "perspective" of the nearest grid axis index.
    /// If the overlap is even *and* if the index is exactly in the middle of the overlap,
    /// `tie_break` is used to determine the nearst grid axis index.
    pub(in crate::world::indexing) fn normalize_to_nearest_world_grid_axis_index(
        self,
        world_grid_axis_count: NonZeroUsize,
        overlap: u8,
        tie_break: AxisOrdering,
    ) -> Self {
        let grid_stride = WorldGridPosition::stride::<Base>(overlap);
        let cell_axis_index = self.cell_axis_index.get();
        let is_even_overlap = overlap.is_even();
        let half_overlap_round_down = overlap / 2;

        let start_overlap_upper_bound_to_before = if is_even_overlap {
            half_overlap_round_down
        } else {
            half_overlap_round_down
                + match tie_break {
                    AxisOrdering::Less => 1,
                    AxisOrdering::Greater => 0,
                }
        };

        let end_overlap_lower_bound_to_after = start_overlap_upper_bound_to_before + grid_stride;

        let neighbor_ordering = if cell_axis_index < start_overlap_upper_bound_to_before {
            // inside start overlap first half => neighbor before
            Some(AxisOrdering::Less)
        } else if cell_axis_index >= end_overlap_lower_bound_to_after {
            // inside end overlap second half => neighbor after
            Some(AxisOrdering::Greater)
        } else {
            // not in any overlap or overlap half does map to self
            None
        };

        neighbor_ordering
            .and_then(|neighbor_ordering| {
                self.to_neighboring_grid_axis_index(
                    neighbor_ordering,
                    world_grid_axis_count,
                    overlap,
                )
            })
            .unwrap_or(self)
    }
}

/// Testing helpers
impl<Base: SudokuBase> WorldGridCellAxisIndex<Base> {
    #[cfg(test)]
    fn all(world_grid_axis_count: NonZeroUsize) -> impl Iterator<Item = Self> {
        (0..world_grid_axis_count.get()).flat_map(|world_grid_axis_index| {
            Coordinate::<Base>::all().map(move |cell_axis_index| WorldGridCellAxisIndex {
                world_grid_axis_index,
                cell_axis_index,
            })
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use crate::base::consts::*;

    #[test]
    fn test_from_world_cell_axis_index() {
        type Base = Base2;
        let world_grid_axis_count = 3.try_into().unwrap();
        let overlap = 2;
        let world_cell_axis_count = 8;

        itertools::assert_equal(
            (0..world_cell_axis_count).map(|world_cell_axis_index| {
                WorldGridCellAxisIndex::<Base>::from_world_cell_axis_index(
                    world_cell_axis_index,
                    world_grid_axis_count,
                    overlap,
                )
            }),
            vec![
                (0, 0),
                (0, 1),
                (1, 0),
                (1, 1),
                (2, 0),
                (2, 1),
                (2, 2),
                (2, 3),
            ]
            .into_iter()
            .map(
                |(world_grid_axis_index, cell_axis_index)| WorldGridCellAxisIndex {
                    world_grid_axis_index,
                    cell_axis_index: Coordinate::new(cell_axis_index).unwrap(),
                },
            ),
        );
    }

    #[test]
    fn test_all() {
        type Base = Base2;

        let world_grid_axis_count = 3.try_into().unwrap();

        itertools::assert_equal(
            WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count),
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
            ]
            .into_iter()
            .map(
                |(world_grid_axis_index, cell_axis_index)| WorldGridCellAxisIndex {
                    world_grid_axis_index,
                    cell_axis_index: Coordinate::new(cell_axis_index).unwrap(),
                },
            ),
        );
    }

    mod overlap_neighbor {

        use super::*;

        fn test_self_inverse<Base: SudokuBase>(overlap: u8) {
            let world_grid_axis_count = 3.try_into().unwrap();

            for world_grid_cell_axis_index in
                WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count)
            {
                let neighbor =
                    world_grid_cell_axis_index.overlap_neighbor(world_grid_axis_count, overlap);
                if let Some((neighbor, order_neighbor)) = neighbor {
                    let (inverse_neighbor, order_inverse_neighbor) = neighbor
                        .overlap_neighbor(world_grid_axis_count, overlap)
                        .unwrap();

                    assert_eq!(world_grid_cell_axis_index, inverse_neighbor);
                    assert_ne!(order_neighbor, order_inverse_neighbor);
                }
            }
        }

        #[test]
        fn test_self_inverse_base2() {
            test_self_inverse::<Base2>(0);
            test_self_inverse::<Base2>(1);
            test_self_inverse::<Base2>(2);
        }
        #[test]
        fn test_self_inverse_base3() {
            test_self_inverse::<Base3>(0);
            test_self_inverse::<Base3>(1);
            test_self_inverse::<Base3>(2);
            test_self_inverse::<Base3>(3);
        }
        #[test]
        fn test_self_inverse_base4() {
            test_self_inverse::<Base4>(0);
            test_self_inverse::<Base4>(1);
            test_self_inverse::<Base4>(2);
            test_self_inverse::<Base4>(3);
            test_self_inverse::<Base4>(4);
        }
    }

    #[test]
    fn test_overlap_neighbor_base2_overlap_0() {
        type Base = Base2;
        let world_grid_axis_count = 3.try_into().unwrap();
        let overlap = 0;

        for world_grid_cell_axis_index in WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count)
        {
            assert!(world_grid_cell_axis_index
                .overlap_neighbor(world_grid_axis_count, overlap)
                .is_none());
        }
    }

    #[test]
    fn test_overlap_neighbor_base2_overlap_1() {
        type Base = Base2;
        let world_grid_axis_count = 3.try_into().unwrap();
        let overlap = 1;

        let expected_results: Vec<Option<(WorldGridCellAxisIndex<Base>, AxisOrdering)>> = vec![
            // Grid 0
            None,
            None,
            None,
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                },
                AxisOrdering::Greater,
            )),
            // Grid 1
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 0,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                },
                AxisOrdering::Less,
            )),
            None,
            None,
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 2,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                },
                AxisOrdering::Greater,
            )),
            // Grid 2
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                },
                AxisOrdering::Less,
            )),
            None,
            None,
            None,
        ];

        itertools::assert_equal(
            WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count).map(
                |world_grid_cell_axis_index| {
                    world_grid_cell_axis_index.overlap_neighbor(world_grid_axis_count, overlap)
                },
            ),
            expected_results,
        );
    }

    #[test]
    fn test_overlap_neighbor_base2_overlap_2() {
        type Base = Base2;
        let world_grid_axis_count = 3.try_into().unwrap();
        let overlap = 2;

        let expected_results: Vec<Option<(WorldGridCellAxisIndex<Base>, AxisOrdering)>> = vec![
            // Grid 0
            None,
            None,
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                },
                AxisOrdering::Greater,
            )),
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(1).unwrap(),
                },
                AxisOrdering::Greater,
            )),
            // Grid 1
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 0,
                    cell_axis_index: Coordinate::new(2).unwrap(),
                },
                AxisOrdering::Less,
            )),
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 0,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                },
                AxisOrdering::Less,
            )),
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 2,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                },
                AxisOrdering::Greater,
            )),
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 2,
                    cell_axis_index: Coordinate::new(1).unwrap(),
                },
                AxisOrdering::Greater,
            )),
            // Grid 2
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(2).unwrap(),
                },
                AxisOrdering::Less,
            )),
            Some((
                WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                },
                AxisOrdering::Less,
            )),
            None,
            None,
        ];

        itertools::assert_equal(
            WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count).map(
                |world_grid_cell_axis_index| {
                    world_grid_cell_axis_index.overlap_neighbor(world_grid_axis_count, overlap)
                },
            ),
            expected_results,
        );
    }
}
