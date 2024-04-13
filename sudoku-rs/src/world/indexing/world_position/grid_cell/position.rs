use std::fmt::Display;

use crate::{
    base::SudokuBase,
    position::Position,
    world::{GridOverlap, WorldCellPosition, WorldGridPosition},
};

use super::WorldGridCellAxisIndex;

/// The position of a cell inside a specific grid in the world.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WorldGridCellPosition<Base: SudokuBase> {
    /// The position of the grid in the world.
    world_grid_pos: WorldGridPosition,
    /// The position of the cell inside this grid.
    cell_pos: Position<Base>,
}

impl<Base: SudokuBase> Display for WorldGridCellPosition<Base> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}>{}", self.world_grid_pos, self.cell_pos)
    }
}

impl<Base: SudokuBase> WorldGridCellPosition<Base> {
    pub fn world_grid_pos(self) -> WorldGridPosition {
        self.world_grid_pos
    }
    pub fn cell_pos(self) -> Position<Base> {
        self.cell_pos
    }

    pub fn to_world_cell_pos(self, overlap: GridOverlap<Base>) -> WorldCellPosition {
        let Self {
            world_grid_pos,
            cell_pos,
        } = self;

        world_grid_pos.to_top_left_cell_position(overlap) + cell_pos
    }
}

impl<Base: SudokuBase> From<(WorldGridPosition, Position<Base>)> for WorldGridCellPosition<Base> {
    fn from((world_grid_pos, cell_pos): (WorldGridPosition, Position<Base>)) -> Self {
        Self {
            world_grid_pos,
            cell_pos,
        }
    }
}

impl<Base: SudokuBase> From<(WorldGridCellAxisIndex<Base>, WorldGridCellAxisIndex<Base>)>
    for WorldGridCellPosition<Base>
{
    fn from((row, column): (WorldGridCellAxisIndex<Base>, WorldGridCellAxisIndex<Base>)) -> Self {
        Self {
            world_grid_pos: WorldGridPosition::new(
                row.world_grid_axis_index(),
                column.world_grid_axis_index(),
            ),
            cell_pos: (row.cell_axis_index(), column.cell_axis_index()).into(),
        }
    }
}
