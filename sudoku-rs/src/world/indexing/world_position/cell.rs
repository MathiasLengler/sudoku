use std::ops::Add;

use crate::{
    base::SudokuBase,
    position::{Coordinate, Position},
    world::{CellMarker, WorldGridCellPosition, WorldGridDim, WorldGridPosition},
};

use super::WorldPosition;

pub type WorldCellPosition = WorldPosition<CellMarker>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct WorldGridCellAxisIndex<Base: SudokuBase> {
    world_grid_axis_index: usize,
    cell_axis_index: Coordinate<Base>,
}

impl<Base: SudokuBase> WorldGridCellAxisIndex<Base> {
    fn all(world_grid_axis_count: usize) -> impl Iterator<Item = Self> {
        (0..world_grid_axis_count).flat_map(|world_grid_axis_index| {
            Coordinate::<Base>::all().map(move |cell_axis_index| WorldGridCellAxisIndex {
                world_grid_axis_index,
                cell_axis_index,
            })
        })
    }

    // TODO: use in
    fn overlap_neighbor(self, world_grid_axis_count: usize, overlap: u8) -> Option<Self> {
        let Self {
            world_grid_axis_index,
            cell_axis_index,
        } = self;

        let grid_stride = WorldGridPosition::stride::<Base>(overlap);

        let cell_axis_index = cell_axis_index.get();

        if cell_axis_index < overlap {
            // inside start overlap
            if world_grid_axis_index == 0 {
                // no grid before
                None
            } else {
                // neighbor before
                Some(Self {
                    world_grid_axis_index: world_grid_axis_index - 1,
                    cell_axis_index: (cell_axis_index + grid_stride).try_into().unwrap(),
                })
            }
        } else if cell_axis_index >= grid_stride {
            // inside end overlap
            if world_grid_axis_index == world_grid_axis_count - 1 {
                // No grid after
                None
            } else {
                // neighbor after
                Some(Self {
                    world_grid_axis_index: world_grid_axis_index + 1,
                    cell_axis_index: (cell_axis_index - grid_stride).try_into().unwrap(),
                })
            }
        } else {
            // not in any overlap
            None
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CellAxisIndexToGridAxisIndexes<Base: SudokuBase> {
    Unambiguous(WorldGridCellAxisIndex<Base>),
    Overlap(WorldGridCellAxisIndex<Base>, WorldGridCellAxisIndex<Base>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WorldCellPositionToGridCellPositionsReturn<Base: SudokuBase> {
    Unambiguous(WorldGridCellPosition<Base>),
    OverlapSide([WorldGridCellPosition<Base>; 2]),
    OverlapCorner([WorldGridCellPosition<Base>; 4]),
}

impl WorldCellPosition {
    pub fn to_grid_cell_positions<Base: SudokuBase>(
        self,
        _grid_dim: WorldGridDim,
        overlap: u8,
    ) -> WorldCellPositionToGridCellPositionsReturn<Base> {
        // TODO: implement

        let _grid_stride = WorldGridPosition::stride::<Base>(overlap);
        todo!();
    }

    fn cell_axis_index_to_world_grid_cell_axis_indexes<Base: SudokuBase>(
        world_cell_axis_index: usize,
        world_grid_axis_count: usize,
        overlap: u8,
    ) -> CellAxisIndexToGridAxisIndexes<Base> {
        todo!();

        // let grid_stride = WorldGridPosition::stride::<Base>(overlap);

        // let world_grid_cell_axis_index = WorldGridCellAxisIndex::<Base> {
        //     world_grid_axis_index: world_cell_axis_index / grid_stride,
        //     cell_axis_index: Coordinate::new(
        //         (world_cell_axis_index % grid_stride).try_into().unwrap(),
        //     )
        //     .unwrap(),
        // };

        // world_grid_cell_axis_index
        //     .cell_axis_index
        //     .contained_in_world_grid_overlap(overlap)
    }

    // TODO: nearest grid position
    //  use case: world map click on cell: which grid to select
    //  requires clicked cell quadrant for odd overlaps as a tie-breaker
}

impl<Base: SudokuBase> Add<Position<Base>> for WorldCellPosition {
    type Output = Self;

    fn add(self, cell_pos: Position<Base>) -> Self::Output {
        let Self { row, column, .. } = self;

        Self::new(
            row + cell_pos.to_row().get_usize(),
            column + cell_pos.to_column().get_usize(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;

    mod world_grid_cell_axis_index {
        use super::*;

        #[test]
        fn test_all() {
            type Base = Base2;

            itertools::assert_equal(
                WorldGridCellAxisIndex::<Base>::all(3),
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
                let world_grid_axis_count = 3;

                for world_grid_cell_axis_index in
                    (0..world_grid_axis_count).flat_map(|world_grid_axis_index| {
                        Coordinate::<Base>::all().map(move |cell_axis_index| {
                            WorldGridCellAxisIndex {
                                world_grid_axis_index,
                                cell_axis_index,
                            }
                        })
                    })
                {
                    let neighbor =
                        world_grid_cell_axis_index.overlap_neighbor(world_grid_axis_count, overlap);
                    if let Some(neighbor) = neighbor {
                        let inverse_neighbor = neighbor
                            .overlap_neighbor(world_grid_axis_count, overlap)
                            .unwrap();

                        assert_eq!(world_grid_cell_axis_index, inverse_neighbor);
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
            let world_grid_axis_count = 3;
            let overlap = 0;

            for world_grid_cell_axis_index in WorldGridCellAxisIndex::<Base>::all(3) {
                assert!(world_grid_cell_axis_index
                    .overlap_neighbor(world_grid_axis_count, overlap)
                    .is_none());
            }
        }

        #[test]
        fn test_overlap_neighbor_base2_overlap_1() {
            type Base = Base2;
            let world_grid_axis_count = 3;
            let overlap = 1;

            let expected_results: Vec<Option<WorldGridCellAxisIndex<Base>>> = vec![
                // Grid 0
                None,
                None,
                None,
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                }),
                // Grid 1
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 0,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                }),
                None,
                None,
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 2,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                }),
                // Grid 2
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                }),
                None,
                None,
                None,
            ];

            itertools::assert_equal(
                WorldGridCellAxisIndex::<Base>::all(3).map(|world_grid_cell_axis_index| {
                    world_grid_cell_axis_index.overlap_neighbor(world_grid_axis_count, overlap)
                }),
                expected_results,
            );
        }

        #[test]
        fn test_overlap_neighbor_base2_overlap_2() {
            type Base = Base2;
            let world_grid_axis_count = 3;
            let overlap = 2;

            let expected_results: Vec<Option<WorldGridCellAxisIndex<Base>>> = vec![
                // Grid 0
                None,
                None,
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                }),
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(1).unwrap(),
                }),
                // Grid 1
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 0,
                    cell_axis_index: Coordinate::new(2).unwrap(),
                }),
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 0,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                }),
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 2,
                    cell_axis_index: Coordinate::new(0).unwrap(),
                }),
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 2,
                    cell_axis_index: Coordinate::new(1).unwrap(),
                }),
                // Grid 2
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(2).unwrap(),
                }),
                Some(WorldGridCellAxisIndex {
                    world_grid_axis_index: 1,
                    cell_axis_index: Coordinate::new(3).unwrap(),
                }),
                None,
                None,
            ];

            itertools::assert_equal(
                WorldGridCellAxisIndex::<Base>::all(3).map(|world_grid_cell_axis_index| {
                    world_grid_cell_axis_index.overlap_neighbor(world_grid_axis_count, overlap)
                }),
                expected_results,
            );
        }
    }
}
