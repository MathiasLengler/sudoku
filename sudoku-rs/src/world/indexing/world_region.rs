use serde::{Deserialize, Serialize};

use crate::world::{WorldCellDim, WorldCellPosition};

/// A rectangular region of cells defined by a start (top-left, inclusive) and end (bottom-right, exclusive) position.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldCellRegion {
    /// Top-left corner (inclusive)
    pub start: WorldCellPosition,
    /// Bottom-right corner (exclusive)
    pub end: WorldCellPosition,
}

impl WorldCellRegion {
    /// Creates a new region from start (inclusive) to end (exclusive).
    pub fn new(start: WorldCellPosition, end: WorldCellPosition) -> Self {
        Self { start, end }
    }

    /// Returns the width (column count) of the region.
    pub fn width(&self) -> usize {
        self.end.column.saturating_sub(self.start.column)
    }

    /// Returns the height (row count) of the region.
    pub fn height(&self) -> usize {
        self.end.row.saturating_sub(self.start.row)
    }

    /// Returns the number of cells in this region.
    pub fn cell_count(&self) -> usize {
        self.width() * self.height()
    }

    /// Clamps this region to fit within the given cell dimensions.
    pub fn clamp_to_dim(&self, cell_dim: WorldCellDim) -> Self {
        let start_row = self.start.row.min(cell_dim.row_count.get());
        let start_column = self.start.column.min(cell_dim.column_count.get());
        let end_row = self.end.row.min(cell_dim.row_count.get());
        let end_column = self.end.column.min(cell_dim.column_count.get());

        Self {
            start: WorldCellPosition::new(start_row, start_column),
            end: WorldCellPosition::new(end_row, end_column),
        }
    }

    /// Returns an iterator over all cell positions in this region.
    pub fn positions(&self) -> impl Iterator<Item = WorldCellPosition> {
        let start_row = self.start.row;
        let end_row = self.end.row;
        let start_column = self.start.column;
        let end_column = self.end.column;

        (start_row..end_row)
            .flat_map(move |row| (start_column..end_column).map(move |column| WorldCellPosition::new(row, column)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_dimensions() {
        let region = WorldCellRegion::new(
            WorldCellPosition::new(2, 3),
            WorldCellPosition::new(5, 7),
        );

        assert_eq!(region.width(), 4);
        assert_eq!(region.height(), 3);
        assert_eq!(region.cell_count(), 12);
    }

    #[test]
    fn test_region_positions() {
        let region = WorldCellRegion::new(
            WorldCellPosition::new(0, 0),
            WorldCellPosition::new(2, 2),
        );

        let positions: Vec<_> = region.positions().collect();
        assert_eq!(positions.len(), 4);
        assert_eq!(positions[0], WorldCellPosition::new(0, 0));
        assert_eq!(positions[1], WorldCellPosition::new(0, 1));
        assert_eq!(positions[2], WorldCellPosition::new(1, 0));
        assert_eq!(positions[3], WorldCellPosition::new(1, 1));
    }

    #[test]
    fn test_clamp_to_dim() {
        let cell_dim = WorldCellDim::new(10, 10).unwrap();
        
        // Region within bounds
        let region = WorldCellRegion::new(
            WorldCellPosition::new(2, 2),
            WorldCellPosition::new(5, 5),
        );
        let clamped = region.clamp_to_dim(cell_dim);
        assert_eq!(clamped.start, WorldCellPosition::new(2, 2));
        assert_eq!(clamped.end, WorldCellPosition::new(5, 5));

        // Region exceeding bounds
        let region = WorldCellRegion::new(
            WorldCellPosition::new(5, 5),
            WorldCellPosition::new(15, 15),
        );
        let clamped = region.clamp_to_dim(cell_dim);
        assert_eq!(clamped.start, WorldCellPosition::new(5, 5));
        assert_eq!(clamped.end, WorldCellPosition::new(10, 10));
    }
}
