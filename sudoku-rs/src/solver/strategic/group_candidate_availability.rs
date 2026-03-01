use crate::base::SudokuBase;
use crate::cell::{Candidates, Value};
use crate::grid::group::CandidatesGroup;
use crate::grid::Grid;
use crate::position::{Coordinate, Position};

/// Axis type for row/column operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Axis {
    Row,
    Column,
}

impl Axis {
    /// Get the other axis.
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Axis::Row => Axis::Column,
            Axis::Column => Axis::Row,
        }
    }

    /// Convert axis coordinates to a position.
    pub fn coordinates_to_pos<Base: SudokuBase>(
        self,
        axis_coordinate: Coordinate<Base>,
        other_axis_coordinate: Coordinate<Base>,
    ) -> Position<Base> {
        match self {
            Axis::Row => Position::from((axis_coordinate, other_axis_coordinate)),
            Axis::Column => Position::from((other_axis_coordinate, axis_coordinate)),
        }
    }
}

/// For a single candidate, where in each group is this candidate set?
///
/// This is the "transposed" view of the grid:
/// - Grid: Position (row, column) → Cell → Candidates
/// - `GroupCandidateAvailability`: Candidate → Group type (row/column/block) → `GroupIndex` → bool is set
///
/// Used by strategies like `XWing` and `GroupIntersection` to efficiently answer questions like:
/// - In which rows/columns/blocks is a specific candidate available?
/// - For a given row, which columns have a specific candidate?
#[derive(Debug, Clone, Default)]
pub struct GroupCandidateAvailability<Base: SudokuBase> {
    /// For each row, which column positions have this candidate?
    pub rows: CandidatesGroup<Base>,
    /// For each column, which row positions have this candidate?
    pub columns: CandidatesGroup<Base>,
    /// For each block, which positions (in row-major order within the block) have this candidate?
    pub row_major_blocks: CandidatesGroup<Base>,
    /// For each block, which positions (in column-major order within the block) have this candidate?
    pub column_major_blocks: CandidatesGroup<Base>,
}

impl<Base: SudokuBase> GroupCandidateAvailability<Base> {
    /// Get the availability for a specific axis (row or column).
    pub fn axis(&self, axis: Axis) -> &CandidatesGroup<Base> {
        match axis {
            Axis::Row => &self.rows,
            Axis::Column => &self.columns,
        }
    }

    /// Insert a candidate at a position into the availability data structure.
    pub fn insert(&mut self, pos: Position<Base>) {
        let row_index = pos.to_column();
        self.rows.get_mut(pos.to_row()).insert(row_index);

        let column_index = pos.to_row();
        self.columns.get_mut(pos.to_column()).insert(column_index);

        let (block, row_major_block_index, column_major_block_index) = pos.to_block_and_indexes();
        self.row_major_blocks
            .get_mut(block)
            .insert(row_major_block_index);
        self.column_major_blocks
            .get_mut(block)
            .insert(column_major_block_index);
    }

    /// Delete a candidate at a position from the availability data structure.
    pub fn delete(&mut self, pos: Position<Base>) {
        let row_index = pos.to_column();
        self.rows.get_mut(pos.to_row()).delete(row_index);

        let column_index = pos.to_row();
        self.columns.get_mut(pos.to_column()).delete(column_index);

        let (block, row_major_block_index, column_major_block_index) = pos.to_block_and_indexes();
        self.row_major_blocks
            .get_mut(block)
            .delete(row_major_block_index);
        self.column_major_blocks
            .get_mut(block)
            .delete(column_major_block_index);
    }
}

/// A collection of `GroupCandidateAvailability` for all candidates.
///
/// This is the main data structure that the strategic solver maintains and passes to strategies.
/// It provides a "transposed" view of the grid's candidates, organized by candidate value.
#[derive(Debug, Clone)]
pub struct StrategicGroupAvailability<Base: SudokuBase> {
    /// One `GroupCandidateAvailability` per candidate value.
    /// Index 0 corresponds to candidate value 1, etc.
    candidates: Vec<GroupCandidateAvailability<Base>>,
}

impl<Base: SudokuBase> Default for StrategicGroupAvailability<Base> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Base: SudokuBase> StrategicGroupAvailability<Base> {
    /// Create a new empty `StrategicGroupAvailability`.
    pub fn new() -> Self {
        Self {
            candidates: vec![
                GroupCandidateAvailability::default();
                usize::from(Base::SIDE_LENGTH)
            ],
        }
    }

    /// Construct a `StrategicGroupAvailability` from a grid.
    pub fn from_grid(grid: &Grid<Base>) -> Self {
        let mut this = Self::new();

        for pos in Position::<Base>::all() {
            if let Some(candidates) = grid[pos].candidates() {
                for candidate in candidates {
                    this.get_mut(candidate).insert(pos);
                }
            }
        }

        this
    }

    /// Get the availability for a specific candidate value.
    pub fn get(&self, candidate: Value<Base>) -> &GroupCandidateAvailability<Base> {
        &self.candidates[usize::from(candidate.get() - 1)]
    }

    /// Get a mutable reference to the availability for a specific candidate value.
    pub fn get_mut(&mut self, candidate: Value<Base>) -> &mut GroupCandidateAvailability<Base> {
        &mut self.candidates[usize::from(candidate.get() - 1)]
    }

    /// Iterate over all candidate availabilities along with their candidate values.
    pub fn iter(&self) -> impl Iterator<Item = (Value<Base>, &GroupCandidateAvailability<Base>)> {
        Value::<Base>::all().zip(self.candidates.iter())
    }

    /// Delete a candidate from a position.
    /// Call this when a deduction removes a candidate from a cell.
    pub fn delete_candidate(&mut self, pos: Position<Base>, candidate: Value<Base>) {
        self.get_mut(candidate).delete(pos);
    }

    /// Delete all candidates from a position.
    /// Call this when a cell is set to a value.
    pub fn delete_all_candidates(&mut self, pos: Position<Base>, candidates: Candidates<Base>) {
        for candidate in candidates {
            self.delete_candidate(pos, candidate);
        }
    }

    /// Update the availability based on setting a value at a position.
    /// This removes all candidates for that position, since it now has a value.
    pub fn set_value(&mut self, pos: Position<Base>, previous_candidates: Candidates<Base>) {
        self.delete_all_candidates(pos, previous_candidates);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;

    #[test]
    fn test_from_grid_empty() {
        let grid = Grid::<Base2>::new();
        let availability = StrategicGroupAvailability::from_grid(&grid);

        // Empty grid has no candidates, so all availability should be empty
        for candidate in Value::<Base2>::all() {
            let ca = availability.get(candidate);
            for coord in crate::position::Coordinate::<Base2>::all() {
                assert!(ca.rows.get(coord).is_empty());
                assert!(ca.columns.get(coord).is_empty());
                assert!(ca.row_major_blocks.get(coord).is_empty());
                assert!(ca.column_major_blocks.get(coord).is_empty());
            }
        }
    }

    #[test]
    fn test_from_grid_with_candidates() {
        let mut grid = Grid::<Base2>::new();
        grid.set_all_direct_candidates();

        let availability = StrategicGroupAvailability::from_grid(&grid);

        // All cells have all candidates, so for each candidate, all positions should be set
        for candidate in Value::<Base2>::all() {
            let ca = availability.get(candidate);
            for coord in crate::position::Coordinate::<Base2>::all() {
                assert_eq!(
                    ca.rows.get(coord),
                    Candidates::all(),
                    "row {coord} should have all candidates"
                );
                assert_eq!(
                    ca.columns.get(coord),
                    Candidates::all(),
                    "column {coord} should have all candidates"
                );
            }
        }
    }

    #[test]
    fn test_delete_candidate() {
        let mut grid = Grid::<Base2>::new();
        grid.set_all_direct_candidates();

        let mut availability = StrategicGroupAvailability::from_grid(&grid);

        let pos: Position<Base2> = (0, 0).try_into().unwrap();
        let candidate: Value<Base2> = 1.try_into().unwrap();

        // Before deletion, candidate 1 should be available at position (0,0)
        assert!(availability.get(candidate).rows.get(pos.to_row()).has(pos.to_column()));

        // Delete candidate 1 from position (0,0)
        availability.delete_candidate(pos, candidate);

        // After deletion, candidate 1 should not be available at position (0,0)
        assert!(!availability.get(candidate).rows.get(pos.to_row()).has(pos.to_column()));
        assert!(!availability.get(candidate).columns.get(pos.to_column()).has(pos.to_row()));
    }
}
