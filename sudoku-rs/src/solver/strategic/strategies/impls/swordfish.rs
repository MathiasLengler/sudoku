use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::grid::group::CandidatesGroup;
use crate::position::Coordinate;
use crate::position::Position;
use crate::solver::strategic::deduction::Action;
use crate::solver::strategic::deduction::Deduction;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::deduction::Reason;
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;
use itertools::izip;

/*
Logic:

Swordfish is an extension of X-Wing. While X-Wing works with 2 rows/columns,
Swordfish works with 3 rows/columns.

For each candidate:

Build subset of `GroupCandidateIndexes` (same as X-Wing):
```
rows: CandidatesGroup<Base>,
columns: CandidatesGroup<Base>,
```

For Swordfish, we are interested in rows or columns with exactly 2 OR 3 candidates set.
We want to find three such rows (or columns) where all candidates fall within the same
three columns (or rows).

Starting with rows, then inversely for columns.

Filter rows with 2 or 3 candidates in the same columns.
Find groups of 3 rows that share the same 3 columns for their candidates.

A Swordfish candidate is identified by:
3 distinct row coordinates
3 distinct column coordinates

Since we arrived at the Swordfish candidate via rows, we now look at the affected columns for eliminations.
Get all three columns, count candidates:
- If only the Swordfish candidate itself, nothing to eliminate.
- If more: Swordfish found.

Candidates to delete: `column_candidates without Swordfish candidates (3 row coordinates)`

Deduction:
`Action::DeleteCandidates(Candidates::with_single(candidate))` - positions: affected columns, excluding Swordfish rows
`Reason::Candidates(Candidates::with_single(candidate))` - the positions forming the Swordfish

Repeat the same logic, starting with columns.
Repeat for all candidates.
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Swordfish;

impl Strategy for Swordfish {
    fn name(self) -> &'static str {
        "Swordfish"
    }

    fn score(self) -> StrategyScore {
        350
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        let candidate_to_group_candidate_indexes = GroupCandidateIndexes::with_grid(grid);

        Ok(izip!(
            Value::<Base>::all(),
            candidate_to_group_candidate_indexes.iter()
        )
        .flat_map(|(candidate, group_candidate_indexes)| {
            find_swordfish(candidate, group_candidate_indexes)
        })
        .collect())
    }
}

fn find_swordfish<Base: SudokuBase>(
    candidate: Value<Base>,
    group_candidate_indexes: &GroupCandidateIndexes<Base>,
) -> impl Iterator<Item = Deduction<Base>> + '_ {
    find_swordfish_axis(candidate, group_candidate_indexes, Axis::Row).chain(find_swordfish_axis(
        candidate,
        group_candidate_indexes,
        Axis::Column,
    ))
}

fn find_swordfish_axis<Base: SudokuBase>(
    candidate: Value<Base>,
    group_candidate_indexes: &GroupCandidateIndexes<Base>,
    axis: Axis,
) -> impl Iterator<Item = Deduction<Base>> + '_ {
    let opposite_axis = axis.other();

    // Find all rows (or columns) where the candidate appears in exactly 2 or 3 positions
    let potential_lines: Vec<_> = group_candidate_indexes
        .axis(axis)
        .iter_enumerate()
        .filter(|(_, candidates)| {
            let count = candidates.count();
            count == 2 || count == 3
        })
        .collect();

    // Group potential lines by their column pattern
    // For Swordfish, we need to find 3 lines whose combined column positions form exactly 3 columns
    let mut swordfish_patterns: Vec<SwordfishPattern<Base>> = Vec::new();

    // Try all combinations of 3 lines
    for i in 0..potential_lines.len() {
        for j in (i + 1)..potential_lines.len() {
            for k in (j + 1)..potential_lines.len() {
                let (coord_i, cols_i) = potential_lines[i];
                let (coord_j, cols_j) = potential_lines[j];
                let (coord_k, cols_k) = potential_lines[k];

                // Combine all column positions
                let combined_cols = cols_i.union(cols_j).union(cols_k);

                // For Swordfish, combined columns must be exactly 3
                if combined_cols.count() == 3 {
                    let mut axis_coordinates = Candidates::new();
                    axis_coordinates.set(coord_i, true);
                    axis_coordinates.set(coord_j, true);
                    axis_coordinates.set(coord_k, true);

                    swordfish_patterns.push(match axis {
                        Axis::Row => {
                            SwordfishPattern::new(candidate, axis_coordinates, combined_cols)
                        }
                        Axis::Column => {
                            SwordfishPattern::new(candidate, combined_cols, axis_coordinates)
                        }
                    });
                }
            }
        }
    }

    swordfish_patterns
        .into_iter()
        .filter_map(move |swordfish_pattern| {
            let candidates_positions_to_delete: Vec<_> = swordfish_pattern
                .axis_coordinates(opposite_axis)
                .into_iter()
                .filter_map(|other_axis_coordinate| {
                    let axis_coordinates = group_candidate_indexes
                        .axis(opposite_axis)
                        .get(other_axis_coordinate.into());

                    let axis_coordinates_to_delete =
                        axis_coordinates.without(swordfish_pattern.axis_coordinates(axis));

                    if axis_coordinates_to_delete.is_empty() {
                        None
                    } else {
                        Some(axis_coordinates_to_delete.into_iter().map(
                            move |axis_coordinate_to_delete| {
                                axis.coordinates_to_pos(
                                    Coordinate::from(axis_coordinate_to_delete),
                                    Coordinate::from(other_axis_coordinate),
                                )
                            },
                        ))
                    }
                })
                .flatten()
                .collect();

            if candidates_positions_to_delete.is_empty() {
                return None;
            }

            Some(
                Deduction::try_from_iters(
                    candidates_positions_to_delete
                        .into_iter()
                        .map(|candidates_position_to_delete| {
                            (
                                candidates_position_to_delete,
                                Action::delete_candidate(swordfish_pattern.candidate),
                            )
                        }),
                    swordfish_pattern
                        .to_positions()
                        .map(|pos| (pos, Reason::candidate(swordfish_pattern.candidate))),
                )
                .unwrap(),
            )
        })
}

/// For a single candidate, where in each group is this candidate set?
#[derive(Debug, Clone, Default)]
struct GroupCandidateIndexes<Base: SudokuBase> {
    rows: CandidatesGroup<Base>,
    columns: CandidatesGroup<Base>,
}

impl<Base: SudokuBase> GroupCandidateIndexes<Base> {
    fn with_grid(grid: &Grid<Base>) -> Vec<Self> {
        let mut candidate_to_group_candidate_indexes =
            vec![GroupCandidateIndexes::<Base>::default(); usize::from(Base::SIDE_LENGTH)];

        for pos in Position::<Base>::all() {
            if let Some(candidates) = grid[pos].candidates() {
                for candidate in candidates {
                    let group_candidate_indexes =
                        &mut candidate_to_group_candidate_indexes[usize::from(candidate.get() - 1)];

                    let row_index = pos.to_column();
                    group_candidate_indexes
                        .rows
                        .get_mut(pos.to_row())
                        .insert(row_index);
                    let column_index = pos.to_row();
                    group_candidate_indexes
                        .columns
                        .get_mut(pos.to_column())
                        .insert(column_index);
                }
            }
        }
        candidate_to_group_candidate_indexes
    }

    fn axis(&self, axis: Axis) -> &CandidatesGroup<Base> {
        match axis {
            Axis::Row => &self.rows,
            Axis::Column => &self.columns,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Axis {
    Row,
    Column,
}

impl Axis {
    fn other(self) -> Self {
        match self {
            Axis::Row => Axis::Column,
            Axis::Column => Axis::Row,
        }
    }

    fn coordinates_to_pos<Base: SudokuBase>(
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

/// A detected Swordfish pattern in a sudoku grid.
///
/// It consists of a candidate in 6-9 cells, forming a 3x3 pattern.
/// In one axis, the candidates of the Swordfish are the only ones remaining in those 3 lines.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct SwordfishPattern<Base: SudokuBase> {
    /// The candidate which forms the Swordfish.
    candidate: Value<Base>,
    /// The three row coordinates of the Swordfish.
    row_coordinates: Candidates<Base>,
    /// The three column coordinates of the Swordfish.
    column_coordinates: Candidates<Base>,
}

impl<Base: SudokuBase> SwordfishPattern<Base> {
    fn new(
        candidate: Value<Base>,
        row_coordinates: Candidates<Base>,
        column_coordinates: Candidates<Base>,
    ) -> Self {
        debug_assert_eq!(row_coordinates.count(), 3);
        debug_assert_eq!(column_coordinates.count(), 3);
        Self {
            candidate,
            row_coordinates,
            column_coordinates,
        }
    }

    fn axis_coordinates(&self, axis: Axis) -> Candidates<Base> {
        match axis {
            Axis::Row => self.row_coordinates,
            Axis::Column => self.column_coordinates,
        }
    }

    /// Iterate over all positions forming the Swordfish pattern (up to 9 positions).
    fn to_positions(self) -> impl Iterator<Item = Position<Base>> {
        self.row_coordinates
            .into_iter()
            .flat_map(move |row_coordinate| {
                self.column_coordinates
                    .into_iter()
                    .map(move |column_coordinate| {
                        Position::from((
                            Coordinate::from(row_coordinate),
                            Coordinate::from(column_coordinate),
                        ))
                    })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::consts::*, cell::Cell};

    mod synthetic {
        use super::*;

        #[test]
        fn test_synthetic_base_3_swordfish_row() {
            // Create a synthetic 9x9 grid with a Swordfish pattern
            // Rows 0, 3, 6 have candidate 1 only in columns 0, 3, 6
            let mut grid: Grid<Base3> = Grid::default();

            // Define Swordfish pattern for candidate 1 (value 1)
            // Row 0: candidate in cols 0, 3, 6
            // Row 3: candidate in cols 0, 3, 6
            // Row 6: candidate in cols 0, 3, 6
            let swordfish_rows = [0u8, 3, 6];
            let swordfish_cols = [0u8, 3, 6];
            let candidate_val = Value::<Base3>::try_from(1u8).unwrap();

            // Initialize all cells with all candidates
            for pos in Position::<Base3>::all() {
                grid[pos] = Cell::with_candidates(Candidates::all());
            }

            // Remove candidate 1 from Swordfish rows except for Swordfish columns
            for &row in &swordfish_rows {
                for col in 0..9u8 {
                    let pos = Position::<Base3>::try_from((row, col)).unwrap();
                    if !swordfish_cols.contains(&col) {
                        let mut candidates = grid[pos].candidates().unwrap();
                        candidates.delete(candidate_val);
                        grid[pos] = Cell::with_candidates(candidates);
                    }
                }
            }

            let deductions = Swordfish.execute(&grid).unwrap();

            // Should find eliminations in columns 0, 3, 6 outside of rows 0, 3, 6
            assert!(!deductions.is_empty(), "Expected deductions but got none");

            // Verify eliminations are in the correct positions
            for deduction in deductions.iter() {
                for (pos, _action) in deduction.actions.iter() {
                    let row = pos.to_row().get();
                    let col = pos.to_column().get();
                    // Should not be in Swordfish rows
                    assert!(
                        !swordfish_rows.contains(&row),
                        "Elimination should not be in Swordfish rows: ({}, {})",
                        row,
                        col
                    );
                    // Should be in Swordfish columns
                    assert!(
                        swordfish_cols.contains(&col),
                        "Elimination should be in Swordfish columns: ({}, {})",
                        row,
                        col
                    );
                }
            }
        }

        #[test]
        fn test_synthetic_base_3_swordfish_column() {
            // Create a synthetic 9x9 grid with a Swordfish pattern (column-based)
            // Columns 1, 4, 7 have candidate 2 only in rows 1, 4, 7
            let mut grid: Grid<Base3> = Grid::default();

            let swordfish_cols = [1u8, 4, 7];
            let swordfish_rows = [1u8, 4, 7];
            let candidate_val = Value::<Base3>::try_from(2u8).unwrap();

            // Initialize all cells with all candidates
            for pos in Position::<Base3>::all() {
                grid[pos] = Cell::with_candidates(Candidates::all());
            }

            // Remove candidate 2 from Swordfish columns except for Swordfish rows
            for &col in &swordfish_cols {
                for row in 0..9u8 {
                    let pos = Position::<Base3>::try_from((row, col)).unwrap();
                    if !swordfish_rows.contains(&row) {
                        let mut candidates = grid[pos].candidates().unwrap();
                        candidates.delete(candidate_val);
                        grid[pos] = Cell::with_candidates(candidates);
                    }
                }
            }

            let deductions = Swordfish.execute(&grid).unwrap();

            // Should find eliminations in rows 1, 4, 7 outside of columns 1, 4, 7
            assert!(!deductions.is_empty(), "Expected deductions but got none");

            // Verify eliminations are in the correct positions
            for deduction in deductions.iter() {
                for (pos, _action) in deduction.actions.iter() {
                    let row = pos.to_row().get();
                    let col = pos.to_column().get();
                    // Should not be in Swordfish columns
                    assert!(
                        !swordfish_cols.contains(&col),
                        "Elimination should not be in Swordfish columns: ({}, {})",
                        row,
                        col
                    );
                    // Should be in Swordfish rows
                    assert!(
                        swordfish_rows.contains(&row),
                        "Elimination should be in Swordfish rows: ({}, {})",
                        row,
                        col
                    );
                }
            }
        }

        #[test]
        fn test_swordfish_with_partial_pattern() {
            // Test Swordfish where each row has only 2 candidates (not all 3)
            // Row 0: candidate in cols 0, 3
            // Row 3: candidate in cols 3, 6
            // Row 6: candidate in cols 0, 6
            // Combined still forms 3 columns: 0, 3, 6
            let mut grid: Grid<Base3> = Grid::default();

            let candidate_val = Value::<Base3>::try_from(3u8).unwrap();

            // Initialize all cells with all candidates
            for pos in Position::<Base3>::all() {
                grid[pos] = Cell::with_candidates(Candidates::all());
            }

            // Row 0: keep only cols 0, 3 for candidate 3
            for col in 0..9u8 {
                let pos = Position::<Base3>::try_from((0u8, col)).unwrap();
                if col != 0 && col != 3 {
                    let mut candidates = grid[pos].candidates().unwrap();
                    candidates.delete(candidate_val);
                    grid[pos] = Cell::with_candidates(candidates);
                }
            }

            // Row 3: keep only cols 3, 6 for candidate 3
            for col in 0..9u8 {
                let pos = Position::<Base3>::try_from((3u8, col)).unwrap();
                if col != 3 && col != 6 {
                    let mut candidates = grid[pos].candidates().unwrap();
                    candidates.delete(candidate_val);
                    grid[pos] = Cell::with_candidates(candidates);
                }
            }

            // Row 6: keep only cols 0, 6 for candidate 3
            for col in 0..9u8 {
                let pos = Position::<Base3>::try_from((6u8, col)).unwrap();
                if col != 0 && col != 6 {
                    let mut candidates = grid[pos].candidates().unwrap();
                    candidates.delete(candidate_val);
                    grid[pos] = Cell::with_candidates(candidates);
                }
            }

            let deductions = Swordfish.execute(&grid).unwrap();

            // Should find eliminations in columns 0, 3, 6 outside of rows 0, 3, 6
            assert!(!deductions.is_empty(), "Expected deductions but got none");

            // Verify eliminations
            let swordfish_rows = [0u8, 3, 6];
            let swordfish_cols = [0u8, 3, 6];
            for deduction in deductions.iter() {
                for (pos, _action) in deduction.actions.iter() {
                    let row = pos.to_row().get();
                    let col = pos.to_column().get();
                    assert!(
                        !swordfish_rows.contains(&row),
                        "Elimination should not be in Swordfish rows: ({}, {})",
                        row,
                        col
                    );
                    assert!(
                        swordfish_cols.contains(&col),
                        "Elimination should be in Swordfish columns: ({}, {})",
                        row,
                        col
                    );
                }
            }
        }

        #[test]
        fn test_no_swordfish_when_pattern_incomplete() {
            // Test that we don't find a Swordfish when the pattern spans 4+ columns
            let mut grid: Grid<Base3> = Grid::default();

            let candidate_val = Value::<Base3>::try_from(4u8).unwrap();

            // Initialize all cells with all candidates
            for pos in Position::<Base3>::all() {
                grid[pos] = Cell::with_candidates(Candidates::all());
            }

            // Row 0: candidate in cols 0, 3
            for col in 0..9u8 {
                let pos = Position::<Base3>::try_from((0u8, col)).unwrap();
                if col != 0 && col != 3 {
                    let mut candidates = grid[pos].candidates().unwrap();
                    candidates.delete(candidate_val);
                    grid[pos] = Cell::with_candidates(candidates);
                }
            }

            // Row 3: candidate in cols 3, 6
            for col in 0..9u8 {
                let pos = Position::<Base3>::try_from((3u8, col)).unwrap();
                if col != 3 && col != 6 {
                    let mut candidates = grid[pos].candidates().unwrap();
                    candidates.delete(candidate_val);
                    grid[pos] = Cell::with_candidates(candidates);
                }
            }

            // Row 6: candidate in cols 1, 7 (different columns - now spans 5 total columns)
            for col in 0..9u8 {
                let pos = Position::<Base3>::try_from((6u8, col)).unwrap();
                if col != 1 && col != 7 {
                    let mut candidates = grid[pos].candidates().unwrap();
                    candidates.delete(candidate_val);
                    grid[pos] = Cell::with_candidates(candidates);
                }
            }

            let deductions = Swordfish.execute(&grid).unwrap();

            // Filter to only candidate 4 deductions
            let candidate_4_deductions: Vec<_> = deductions
                .iter()
                .filter(|d| {
                    d.reasons.iter().any(|(_, reason)| {
                        let Reason::Candidates(c) = reason;
                        c.has(candidate_val)
                    })
                })
                .collect();

            // Should NOT find any Swordfish deductions for candidate 4
            assert!(
                candidate_4_deductions.is_empty(),
                "Should not find Swordfish when pattern spans more than 3 columns"
            );
        }
    }
}
