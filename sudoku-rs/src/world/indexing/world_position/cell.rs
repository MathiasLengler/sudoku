use std::ops::Add;

use crate::{
    base::SudokuBase,
    position::Position,
    world::{CellMarker, WorldGridCellPosition, WorldGridDim},
};

use super::WorldPosition;

pub type WorldCellPosition = WorldPosition<CellMarker>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WorldCellPositionToGridCellPositionsReturn<Base: SudokuBase> {
    Unambiguous(WorldGridCellPosition<Base>),
    OverlapSide([WorldGridCellPosition<Base>; 2]),
    OverlapCorner([WorldGridCellPosition<Base>; 4]),
}
impl WorldCellPosition {
    pub fn to_grid_cell_positions<Base: SudokuBase>(
        self,
        grid_dim: WorldGridDim,
    ) -> WorldCellPositionToGridCellPositionsReturn<Base> {
        // TODO: implement
        todo!()
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
