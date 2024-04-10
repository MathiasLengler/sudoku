use std::num::NonZeroUsize;

use num::Integer;

use crate::{
    base::SudokuBase,
    position::Coordinate,
    world::{AxisOrdering, WorldGridPosition},
};

/// The axis index (row or column) of a cell inside a specific grid in the world.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
            // inside potential start overlap => neighbor before
            Some(AxisOrdering::Less)
        } else if cell_axis_index >= grid_stride {
            // inside potential end overlap => neighbor after
            Some(AxisOrdering::Greater)
        } else {
            // not in any potential overlap
            debug_assert!(!Self::is_cell_axis_index_in_potential_overlap_region(
                self.cell_axis_index,
                overlap
            ));
            None
        };

        neighbor_ordering.and_then(|neighbor_ordering| {
            self.to_neighboring_grid_axis_index(neighbor_ordering, world_grid_axis_count, overlap)
                .map(|neighbor| (neighbor, neighbor_ordering))
        })
    }

    /// Normalize `self` to the nearest grid axis index.
    ///
    /// # Behaviour
    ///
    /// If the index is outside of an overlap, returns `self` unchanged.
    ///
    /// If the index is inside an overlap, returns the same index but from the "perspective" of the nearest grid axis index.
    ///
    /// If the overlap is even *and* if the index is exactly in the middle of the overlap,
    /// `tie_break` is used to determine the nearst grid axis index.
    ///
    /// # Use-case
    /// Click on a cell in the world map to select the grid.
    /// Tie break is determined by the clicked cell quadrant.
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

/// Internal helpers
impl<Base: SudokuBase> WorldGridCellAxisIndex<Base> {
    /// Preconditions:
    /// - `WorldGridCellAxisIndex` is inside a potential overlap region
    /// - the overlap region matches the given `neighbor_ordering`
    fn to_neighboring_grid_axis_index(
        self,
        neighbor_ordering: AxisOrdering,
        world_grid_axis_count: NonZeroUsize,
        overlap: u8,
    ) -> Option<Self> {
        debug_assert!(Self::is_cell_axis_index_in_potential_overlap_region(
            self.cell_axis_index,
            overlap
        ));

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

    fn is_cell_axis_index_in_potential_overlap_region(
        cell_axis_index: Coordinate<Base>,
        overlap: u8,
    ) -> bool {
        let grid_stride = WorldGridPosition::stride::<Base>(overlap);

        !(overlap..grid_stride).contains(&cell_axis_index.get())
    }

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

    fn expected_tuples_to_world_grid_cell_axis_indexes<Base: SudokuBase>(
        expected: impl IntoIterator<Item = (usize, u8)>,
    ) -> impl Iterator<Item = WorldGridCellAxisIndex<Base>> {
        expected
            .into_iter()
            .map(
                |(world_grid_axis_index, cell_axis_index)| WorldGridCellAxisIndex {
                    world_grid_axis_index,
                    cell_axis_index: Coordinate::new(cell_axis_index).unwrap(),
                },
            )
    }

    mod public {
        use super::*;

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

        mod overlap_neighbor {
            use super::*;

            mod invariant {
                use super::*;
                fn assert_invariants<Base: SudokuBase>() {
                    // TODO: WorldOverlap::all
                    for overlap in 0..=Base::BASE {
                        let world_grid_axis_count = 3.try_into().unwrap();

                        for world_grid_cell_axis_index in
                            WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count)
                        {
                            // invariant: self inverse
                            let neighbor = world_grid_cell_axis_index
                                .overlap_neighbor(world_grid_axis_count, overlap);
                            if let Some((neighbor, order_neighbor)) = neighbor {
                                let (inverse_neighbor, order_inverse_neighbor) = neighbor
                                    .overlap_neighbor(world_grid_axis_count, overlap)
                                    .unwrap();

                                assert_eq!(world_grid_cell_axis_index, inverse_neighbor);
                                assert_ne!(order_neighbor, order_inverse_neighbor);
                            }
                        }
                    }
                }

                #[test]
                fn base2() {
                    assert_invariants::<Base2>();
                }
                #[test]
                fn base3() {
                    assert_invariants::<Base3>();
                }
                #[test]
                fn base4() {
                    assert_invariants::<Base4>();
                }
                #[test]
                fn base5() {
                    assert_invariants::<Base5>();
                }
            }

            #[test]
            fn test_base2_overlap_0() {
                type Base = Base2;
                let world_grid_axis_count = 3.try_into().unwrap();
                let overlap = 0;

                for world_grid_cell_axis_index in
                    WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count)
                {
                    assert!(world_grid_cell_axis_index
                        .overlap_neighbor(world_grid_axis_count, overlap)
                        .is_none());
                }
            }

            #[test]
            fn test_base2_overlap_1() {
                type Base = Base2;
                let world_grid_axis_count = 3.try_into().unwrap();
                let overlap = 1;

                let expected_results: Vec<Option<(WorldGridCellAxisIndex<Base>, AxisOrdering)>> = vec![
                    // grid 0
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
                    // grid 1
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
                    // grid 2
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
                            world_grid_cell_axis_index
                                .overlap_neighbor(world_grid_axis_count, overlap)
                        },
                    ),
                    expected_results,
                );
            }

            #[test]
            fn test_base2_overlap_2() {
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
                            world_grid_cell_axis_index
                                .overlap_neighbor(world_grid_axis_count, overlap)
                        },
                    ),
                    expected_results,
                );
            }
        }

        mod normalize_to_nearest_world_grid_axis_index {
            use super::*;

            mod invariants {
                use itertools::Itertools;

                use super::*;

                fn assert_invariants<Base: SudokuBase>() {
                    // TODO: WorldOverlap::all
                    for overlap in 0..=Base::BASE {
                        let world_grid_axis_count = 3.try_into().unwrap();

                        for tie_break in AxisOrdering::all() {
                            for world_grid_cell_axis_index in
                                WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count)
                            {
                                let world_grid_cell_axis_index_normalized =
                                    world_grid_cell_axis_index
                                        .normalize_to_nearest_world_grid_axis_index(
                                            world_grid_axis_count,
                                            overlap,
                                            tie_break,
                                        );

                                // invariant: overlap 0 maps to itself
                                if overlap == 0 {
                                    assert_eq!(
                                        world_grid_cell_axis_index_normalized,
                                        world_grid_cell_axis_index
                                    );
                                }

                                // invariant: non potential overlap region maps to itself
                                if !WorldGridCellAxisIndex::<Base>::is_cell_axis_index_in_potential_overlap_region(
                                    world_grid_cell_axis_index.cell_axis_index,
                                    overlap,
                                ) {
                                    assert_eq!(
                                        world_grid_cell_axis_index_normalized,
                                        world_grid_cell_axis_index,
                                    );
                                }

                                // invariant: a normalized position maps to itself
                                assert_eq!(
                                    world_grid_cell_axis_index_normalized
                                        .normalize_to_nearest_world_grid_axis_index(
                                            world_grid_axis_count,
                                            overlap,
                                            tie_break,
                                        ),
                                    world_grid_cell_axis_index_normalized
                                );
                            }

                            // invariant: number of distinct normalized results
                            let grid_stride_usize =
                                WorldGridPosition::stride_usize::<Base>(overlap);
                            assert_eq!(
                                WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count)
                                    .map(|world_grid_cell_axis_index| {
                                        world_grid_cell_axis_index
                                            .normalize_to_nearest_world_grid_axis_index(
                                                world_grid_axis_count,
                                                overlap,
                                                tie_break,
                                            )
                                    })
                                    .unique()
                                    .count(),
                                grid_stride_usize * world_grid_axis_count.get()
                                    + usize::from(overlap)
                            );

                            // invariant: the overlap region shared by two subsequent grids results in the same normalized result
                            for (in_potential_overlap_region, world_grid_cell_axis_indexes) in &WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count).group_by(|world_grid_cell_axis_index| {
                                WorldGridCellAxisIndex::<Base>::is_cell_axis_index_in_potential_overlap_region(
                                    world_grid_cell_axis_index.cell_axis_index,
                                    overlap,
                                )
                            }) {
                                if !in_potential_overlap_region {
                                    continue;
                                }
                                let world_grid_cell_axis_indexes = world_grid_cell_axis_indexes.map(|world_grid_cell_axis_index| {
                                    world_grid_cell_axis_index
                                        .normalize_to_nearest_world_grid_axis_index(
                                            world_grid_axis_count,
                                            overlap,
                                            tie_break,
                                        )
                                }).collect_vec();
                                if world_grid_cell_axis_indexes.len() == usize::from(overlap) * 2 {
                                    let (first_grid_end_overlap, second_grid_start_overlap) = world_grid_cell_axis_indexes.split_at(overlap.into());
                                    assert_eq!(first_grid_end_overlap, second_grid_start_overlap);
                                }
                            }
                        }

                        // invariant: if overlap is even, tie_break does not change the result
                        if overlap.is_even() {
                            for world_grid_cell_axis_index in
                                WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count)
                            {
                                assert_eq!(
                                    world_grid_cell_axis_index
                                        .normalize_to_nearest_world_grid_axis_index(
                                            world_grid_axis_count,
                                            overlap,
                                            AxisOrdering::Less,
                                        ),
                                    world_grid_cell_axis_index
                                        .normalize_to_nearest_world_grid_axis_index(
                                            world_grid_axis_count,
                                            overlap,
                                            AxisOrdering::Greater,
                                        )
                                );
                            }
                        }
                    }
                }

                #[test]
                fn base2() {
                    assert_invariants::<Base2>();
                }
                #[test]
                fn base3() {
                    assert_invariants::<Base3>();
                }
                #[test]
                fn base4() {
                    assert_invariants::<Base4>();
                }
                #[test]
                fn base5() {
                    assert_invariants::<Base5>();
                }
            }

            fn assert_normalize_to_nearest_world_grid_axis_index<Base: SudokuBase>(
                overlap: u8,
                tie_break: AxisOrdering,
                expected: impl IntoIterator<Item = (usize, u8)>,
            ) {
                let world_grid_axis_count = 3.try_into().unwrap();

                itertools::assert_equal(
                    WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count).map(
                        |world_grid_cell_axis_index| {
                            world_grid_cell_axis_index.normalize_to_nearest_world_grid_axis_index(
                                world_grid_axis_count,
                                overlap,
                                tie_break,
                            )
                        },
                    ),
                    expected_tuples_to_world_grid_cell_axis_indexes(expected),
                );
            }

            #[test]
            fn test_base2_overlap_1() {
                type Base = Base2;
                let overlap = 1;

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Less,
                    vec![
                        // grid 0
                        (0, 0),
                        (0, 1),
                        (0, 2),
                        (0, 3),
                        // grid 1
                        (0, 3),
                        (1, 1),
                        (1, 2),
                        (1, 3),
                        // grid 2
                        (1, 3),
                        (2, 1),
                        (2, 2),
                        (2, 3),
                    ],
                );

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Greater,
                    vec![
                        // grid 0
                        (0, 0),
                        (0, 1),
                        (0, 2),
                        (1, 0),
                        // grid 1
                        (1, 0),
                        (1, 1),
                        (1, 2),
                        (2, 0),
                        // grid 2
                        (2, 0),
                        (2, 1),
                        (2, 2),
                        (2, 3),
                    ],
                );
            }

            #[test]
            fn test_base2_overlap_2() {
                type Base = Base2;
                let overlap = 2;

                let expected = vec![
                    // grid 0
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (1, 1),
                    // grid 1
                    (0, 2),
                    (1, 1),
                    (1, 2),
                    (2, 1),
                    // grid 2
                    (1, 2),
                    (2, 1),
                    (2, 2),
                    (2, 3),
                ];

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Less,
                    expected.clone(),
                );

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Greater,
                    expected,
                );
            }

            #[test]
            fn test_base3_overlap_2() {
                type Base = Base3;
                let overlap = 2;

                let expected = vec![
                    // grid 0
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (0, 3),
                    (0, 4),
                    (0, 5),
                    (0, 6),
                    (0, 7),
                    (1, 1),
                    // grid 1
                    (0, 7),
                    (1, 1),
                    (1, 2),
                    (1, 3),
                    (1, 4),
                    (1, 5),
                    (1, 6),
                    (1, 7),
                    (2, 1),
                    // grid 2
                    (1, 7),
                    (2, 1),
                    (2, 2),
                    (2, 3),
                    (2, 4),
                    (2, 5),
                    (2, 6),
                    (2, 7),
                    (2, 8),
                ];

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Less,
                    expected.clone(),
                );

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Greater,
                    expected,
                );
            }

            #[test]
            fn test_base3_overlap_3() {
                type Base = Base3;
                let overlap = 3;

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Less,
                    vec![
                        // grid 0
                        (0, 0),
                        (0, 1),
                        (0, 2),
                        (0, 3),
                        (0, 4),
                        (0, 5),
                        (0, 6),
                        (0, 7),
                        (1, 2),
                        // grid 1
                        (0, 6),
                        (0, 7),
                        (1, 2),
                        (1, 3),
                        (1, 4),
                        (1, 5),
                        (1, 6),
                        (1, 7),
                        (2, 2),
                        // grid 2
                        (1, 6),
                        (1, 7),
                        (2, 2),
                        (2, 3),
                        (2, 4),
                        (2, 5),
                        (2, 6),
                        (2, 7),
                        (2, 8),
                    ],
                );

                assert_normalize_to_nearest_world_grid_axis_index::<Base>(
                    overlap,
                    AxisOrdering::Greater,
                    vec![
                        // grid 0
                        (0, 0),
                        (0, 1),
                        (0, 2),
                        (0, 3),
                        (0, 4),
                        (0, 5),
                        (0, 6),
                        (1, 1),
                        (1, 2),
                        // grid 1
                        (0, 6),
                        (1, 1),
                        (1, 2),
                        (1, 3),
                        (1, 4),
                        (1, 5),
                        (1, 6),
                        (2, 1),
                        (2, 2),
                        // grid 2
                        (1, 6),
                        (2, 1),
                        (2, 2),
                        (2, 3),
                        (2, 4),
                        (2, 5),
                        (2, 6),
                        (2, 7),
                        (2, 8),
                    ],
                );
            }
        }
    }
    mod internal {
        use super::*;
        mod to_neighboring_grid_axis_index {
            use super::*;

            #[test]
            fn test_base2_overlap_1() {
                type Base = Base2;
                let overlap = 1;
                let world_grid_axis_count = 3.try_into().unwrap();

                assert_eq!(
                    WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 0,
                        cell_axis_index: Coordinate::default(),
                    }
                    .to_neighboring_grid_axis_index(
                        AxisOrdering::Less,
                        world_grid_axis_count,
                        overlap,
                    ),
                    None
                );
                assert_eq!(
                    WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 0,
                        cell_axis_index: Coordinate::max(),
                    }
                    .to_neighboring_grid_axis_index(
                        AxisOrdering::Greater,
                        world_grid_axis_count,
                        overlap,
                    ),
                    Some(WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 1,
                        cell_axis_index: Coordinate::default(),
                    })
                );

                assert_eq!(
                    WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 1,
                        cell_axis_index: Coordinate::default(),
                    }
                    .to_neighboring_grid_axis_index(
                        AxisOrdering::Less,
                        world_grid_axis_count,
                        overlap,
                    ),
                    Some(WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 0,
                        cell_axis_index: Coordinate::max(),
                    })
                );
                assert_eq!(
                    WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 1,
                        cell_axis_index: Coordinate::max(),
                    }
                    .to_neighboring_grid_axis_index(
                        AxisOrdering::Greater,
                        world_grid_axis_count,
                        overlap,
                    ),
                    Some(WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 2,
                        cell_axis_index: Coordinate::default(),
                    })
                );
                assert_eq!(
                    WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 2,
                        cell_axis_index: Coordinate::default(),
                    }
                    .to_neighboring_grid_axis_index(
                        AxisOrdering::Less,
                        world_grid_axis_count,
                        overlap,
                    ),
                    Some(WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 1,
                        cell_axis_index: Coordinate::max(),
                    })
                );
                assert_eq!(
                    WorldGridCellAxisIndex::<Base> {
                        world_grid_axis_index: 2,
                        cell_axis_index: Coordinate::max(),
                    }
                    .to_neighboring_grid_axis_index(
                        AxisOrdering::Greater,
                        world_grid_axis_count,
                        overlap,
                    ),
                    None
                );
            }
        }

        mod is_cell_axis_index_in_potential_overlap_region {
            use super::*;

            fn assert_is_cell_axis_index_in_potential_overlap_region<Base: SudokuBase>(
                overlap: u8,
                expected: impl IntoIterator<Item = bool>,
            ) {
                itertools::assert_equal(
                    Coordinate::<Base>::all().map(|cell_axis_index| {
                        WorldGridCellAxisIndex::<Base>::is_cell_axis_index_in_potential_overlap_region(
                            cell_axis_index,
                            overlap,
                        )
                    }),
                    expected,
                );
            }

            #[test]
            fn test_base2_overlap_0() {
                type Base = Base2;
                let overlap = 0;

                assert_is_cell_axis_index_in_potential_overlap_region::<Base>(
                    overlap,
                    vec![false, false, false, false],
                );
            }

            #[test]
            fn test_base2_overlap_1() {
                type Base = Base2;
                let overlap = 1;

                assert_is_cell_axis_index_in_potential_overlap_region::<Base>(
                    overlap,
                    vec![true, false, false, true],
                );
            }
            #[test]
            fn test_base2_overlap_2() {
                type Base = Base2;
                let overlap = 2;

                assert_is_cell_axis_index_in_potential_overlap_region::<Base>(
                    overlap,
                    vec![true, true, true, true],
                );
            }
            #[test]
            fn test_base3_overlap_2() {
                type Base = Base3;
                let overlap = 2;

                assert_is_cell_axis_index_in_potential_overlap_region::<Base>(
                    overlap,
                    vec![true, true, false, false, false, false, false, true, true],
                );
            }
        }

        #[test]
        fn test_all() {
            type Base = Base2;

            let world_grid_axis_count = 3.try_into().unwrap();

            itertools::assert_equal(
                WorldGridCellAxisIndex::<Base>::all(world_grid_axis_count),
                expected_tuples_to_world_grid_cell_axis_indexes(vec![
                    // grid 0
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (0, 3),
                    // grid 1
                    (1, 0),
                    (1, 1),
                    (1, 2),
                    (1, 3),
                    // grid 2
                    (2, 0),
                    (2, 1),
                    (2, 2),
                    (2, 3),
                ]),
            );
        }
    }
}
