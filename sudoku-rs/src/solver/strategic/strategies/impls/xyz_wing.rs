use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::deduction::Action;
use crate::solver::strategic::deduction::Deduction;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::deduction::Reason;
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;

/*
XYZ-Wing strategy:

An XYZ-Wing consists of three cells:
- A "pivot" cell with exactly 3 candidates {X, Y, Z}
- Two "wing" cells, each with exactly 2 candidates:
  - Wing 1: {X, Z} (subset of pivot)
  - Wing 2: {Y, Z} (subset of pivot)
- Each wing must "see" (share a row, column, or block with) the pivot.
- The wings do NOT need to see each other.

The candidate Z (the one shared by all three cells) can be eliminated from
any cell that sees all three cells (pivot + both wings) and contains Z.

Reference: https://www.sudokuwiki.org/XYZ_Wing
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct XyzWing;

impl Strategy for XyzWing {
    fn name(self) -> &'static str {
        "XyzWing"
    }

    fn score(self) -> StrategyScore {
        300
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(find_xyz_wings(grid))
    }
}

fn find_xyz_wings<Base: SudokuBase>(grid: &Grid<Base>) -> Deductions<Base> {
    let candidates_positions = grid.all_candidates_positions();

    // Collect cells with exactly 2 candidates (potential wings)
    let bi_value_cells: Vec<_> = candidates_positions
        .iter()
        .filter_map(|&pos| {
            let candidates = grid[pos].candidates()?;
            (candidates.count() == 2).then_some((pos, candidates))
        })
        .collect();

    // Collect cells with exactly 3 candidates (potential pivots)
    let tri_value_cells: Vec<_> = candidates_positions
        .iter()
        .filter_map(|&pos| {
            let candidates = grid[pos].candidates()?;
            (candidates.count() == 3).then_some((pos, candidates))
        })
        .collect();

    let mut deductions_vec = Vec::new();

    for &(pivot_pos, pivot_candidates) in &tri_value_cells {
        // Find all bi-value cells that see the pivot and are subsets of pivot's candidates
        let wings: Vec<_> = bi_value_cells
            .iter()
            .filter(|&&(wing_pos, wing_candidates)| {
                wing_pos != pivot_pos
                    && wing_candidates.without(pivot_candidates).is_empty()
                    && sees_each_other::<Base>(pivot_pos, wing_pos)
            })
            .copied()
            .collect();

        // Try all pairs of wings
        for i in 0..wings.len() {
            let (wing1_pos, wing1_candidates) = wings[i];
            for &(wing2_pos, wing2_candidates) in &wings[(i + 1)..] {

                // Wings must have different candidate sets to form a valid XYZ-Wing
                if wing1_candidates == wing2_candidates {
                    continue;
                }

                // All three cells together must cover all pivot candidates
                let combined = wing1_candidates.union(wing2_candidates);
                if combined != pivot_candidates {
                    continue;
                }

                // Z is the candidate shared by all three cells
                let z_candidates = pivot_candidates
                    .intersection(wing1_candidates)
                    .intersection(wing2_candidates);

                let Some(z) = z_candidates.to_single() else {
                    continue;
                };

                // Find cells that see all three (pivot + both wings) and contain Z
                if let Some(deduction) =
                    build_deduction(grid, pivot_pos, wing1_pos, wing2_pos, pivot_candidates, z)
                {
                    deductions_vec.push(deduction);
                }
            }
        }
    }

    deductions_vec.into_iter().collect()
}

/// Returns true if two positions share at least one group (row, column, or block).
fn sees_each_other<Base: SudokuBase>(a: Position<Base>, b: Position<Base>) -> bool {
    a.to_row() == b.to_row() || a.to_column() == b.to_column() || a.to_block() == b.to_block()
}

fn build_deduction<Base: SudokuBase>(
    grid: &Grid<Base>,
    pivot_pos: Position<Base>,
    wing1_pos: Position<Base>,
    wing2_pos: Position<Base>,
    pivot_candidates: Candidates<Base>,
    z: Value<Base>,
) -> Option<Deduction<Base>> {
    // Find cells that see all three cells and contain candidate Z
    let positions_to_delete: Vec<_> = grid
        .all_candidates_positions()
        .into_iter()
        .filter(|&pos| {
            pos != pivot_pos
                && pos != wing1_pos
                && pos != wing2_pos
                && sees_each_other::<Base>(pos, pivot_pos)
                && sees_each_other::<Base>(pos, wing1_pos)
                && sees_each_other::<Base>(pos, wing2_pos)
                && grid[pos]
                    .candidates()
                    .is_some_and(|candidates| candidates.has(z))
        })
        .collect();

    if positions_to_delete.is_empty() {
        return None;
    }

    Some(
        Deduction::try_from_iters(
            positions_to_delete
                .into_iter()
                .map(|pos| (pos, Action::delete_candidate(z))),
            [
                (pivot_pos, Reason::candidates(pivot_candidates)),
                (
                    wing1_pos,
                    Reason::candidates(grid[wing1_pos].candidates().unwrap()),
                ),
                (
                    wing2_pos,
                    Reason::candidates(grid[wing2_pos].candidates().unwrap()),
                ),
            ],
        )
        .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::consts::*,
        cell::Cell,
        solver::strategic::strategies::test_util::strategy_snapshot_tests,
    };

    /// Helper to create a position.
    fn pos<Base: SudokuBase>(row: u8, col: u8) -> Position<Base> {
        (row, col).try_into().unwrap()
    }

    /// Helper to create candidates from u8 values.
    fn cands<Base: SudokuBase>(values: &[u8]) -> Candidates<Base> {
        values
            .iter()
            .map(|&v| Value::<Base>::try_from(v).unwrap())
            .collect()
    }

    mod synthetic {
        use super::*;

        #[test]
        fn test_base_3_xyz_wing_row_column() {
            // Construct a Base3 grid with an XYZ-Wing pattern:
            // Pivot at (4,4) with candidates {3,4,9}
            // Wing1 at (4,3) with candidates {3,9} (same row as pivot)
            // Wing2 at (3,4) with candidates {4,9} (same column as pivot)
            // Z = 9 (shared by all three)
            // Target at (3,3): sees pivot (same block 1,1), wing1 (same block 1,1), wing2 (same row 3)
            //
            // Block layout (Base3, 3x3 blocks):
            //   Block (1,1): rows 3-5, cols 3-5
            //   pivot (4,4) -> block (1,1) ✓
            //   wing1 (4,3) -> block (1,1) ✓
            //   wing2 (3,4) -> block (1,1) ✓
            //   target (3,3) -> block (1,1) ✓

            let mut grid = Grid::<Base3>::default();

            let pivot = pos::<Base3>(4, 4);
            let wing1 = pos::<Base3>(4, 3);
            let wing2 = pos::<Base3>(3, 4);
            let target = pos::<Base3>(3, 3);

            grid[pivot] = Cell::with_candidates(cands::<Base3>(&[3, 4, 9]));
            grid[wing1] = Cell::with_candidates(cands::<Base3>(&[3, 9]));
            grid[wing2] = Cell::with_candidates(cands::<Base3>(&[4, 9]));
            grid[target] = Cell::with_candidates(cands::<Base3>(&[1, 5, 9]));

            // Fill other cells with non-interfering candidates (no tri-value cells)
            for p in Position::<Base3>::all() {
                if p != pivot && p != wing1 && p != wing2 && p != target {
                    grid[p] = Cell::with_candidates(cands::<Base3>(&[1, 2]));
                }
            }

            let deductions = XyzWing.execute(&grid).unwrap();

            // Verify: 9 should be eliminated from target (3,3)
            let found = deductions.iter().any(|d| {
                d.actions.iter().any(|(p, action)| {
                    p == target
                        && *action
                            == Action::delete_candidate(Value::<Base3>::try_from(9_u8).unwrap())
                })
            });
            assert!(
                found,
                "Expected XYZ-Wing to eliminate 9 from (3,3). Got: {deductions}"
            );
        }

        #[test]
        fn test_base_3_xyz_wing_block_and_row() {
            // Pivot at (3,3) with candidates {2,5,7}
            // Wing1 at (3,6) with candidates {5,7} (same row as pivot)
            // Wing2 at (4,4) with candidates {2,7} (same block (1,1) as pivot)
            // Z = 7
            //
            // Target at (4,3): sees pivot (same col 3? No, same block (1,1)),
            //   wing1 (4,3) row 4 != row 3, col 3 != col 6, block (1,1) != block (1,2) -> NO
            //
            // Let me recalculate. Cells that see all three:
            //   Must see pivot (3,3) in block (1,1)
            //   Must see wing1 (3,6) in block (1,2)
            //   Must see wing2 (4,4) in block (1,1)
            //
            // (3,4): row 3 = row 3 (pivot ✓), row 3 = row 3 (wing1 ✓), block (1,1) = block (1,1) (wing2 ✓) -> sees all ✓
            // (3,5): row 3 = row 3 (pivot ✓), row 3 = row 3 (wing1 ✓), block (1,1) = block (1,1) (wing2 ✓) -> sees all ✓

            let mut grid = Grid::<Base3>::default();

            let pivot = pos::<Base3>(3, 3);
            let wing1 = pos::<Base3>(3, 6);
            let wing2 = pos::<Base3>(4, 4);
            let target1 = pos::<Base3>(3, 4);
            let target2 = pos::<Base3>(3, 5);

            grid[pivot] = Cell::with_candidates(cands::<Base3>(&[2, 5, 7]));
            grid[wing1] = Cell::with_candidates(cands::<Base3>(&[5, 7]));
            grid[wing2] = Cell::with_candidates(cands::<Base3>(&[2, 7]));
            grid[target1] = Cell::with_candidates(cands::<Base3>(&[1, 7]));
            grid[target2] = Cell::with_candidates(cands::<Base3>(&[3, 7]));

            // Fill other cells with non-interfering candidates
            for p in Position::<Base3>::all() {
                if p != pivot && p != wing1 && p != wing2 && p != target1 && p != target2 {
                    grid[p] = Cell::with_candidates(cands::<Base3>(&[1, 2]));
                }
            }

            let deductions = XyzWing.execute(&grid).unwrap();

            let z_val = Value::<Base3>::try_from(7_u8).unwrap();

            let found_t1 = deductions.iter().any(|d| {
                d.actions
                    .iter()
                    .any(|(p, action)| p == target1 && *action == Action::delete_candidate(z_val))
            });
            let found_t2 = deductions.iter().any(|d| {
                d.actions
                    .iter()
                    .any(|(p, action)| p == target2 && *action == Action::delete_candidate(z_val))
            });

            assert!(
                found_t1,
                "Expected XYZ-Wing to eliminate 7 from (3,4). Got: {deductions}"
            );
            assert!(
                found_t2,
                "Expected XYZ-Wing to eliminate 7 from (3,5). Got: {deductions}"
            );
        }

        #[test]
        fn test_no_xyz_wing_when_no_pattern() {
            // Grid with no XYZ-Wing pattern (all cells have 2 candidates)
            let mut grid = Grid::<Base3>::default();
            for p in Position::<Base3>::all() {
                grid[p] = Cell::with_candidates(cands::<Base3>(&[1, 2]));
            }

            let deductions = XyzWing.execute(&grid).unwrap();
            assert!(
                deductions.is_empty(),
                "Expected no deductions, got: {deductions}"
            );
        }

        #[test]
        fn test_no_elimination_when_no_common_peer() {
            // XYZ-Wing exists but no cell sees all three
            // Pivot at (0,0) with {1,2,3}
            // Wing1 at (0,8) with {1,3} (same row, different block)
            // Wing2 at (8,0) with {2,3} (same column, different block)
            // Z = 3
            // No cell can see (0,0), (0,8), and (8,0) simultaneously in a 9x9 grid

            let mut grid = Grid::<Base3>::default();

            let pivot = pos::<Base3>(0, 0);
            let wing1 = pos::<Base3>(0, 8);
            let wing2 = pos::<Base3>(8, 0);

            grid[pivot] = Cell::with_candidates(cands::<Base3>(&[1, 2, 3]));
            grid[wing1] = Cell::with_candidates(cands::<Base3>(&[1, 3]));
            grid[wing2] = Cell::with_candidates(cands::<Base3>(&[2, 3]));

            // Fill rest with non-interfering candidates
            for p in Position::<Base3>::all() {
                if p != pivot && p != wing1 && p != wing2 {
                    grid[p] = Cell::with_candidates(cands::<Base3>(&[4, 5]));
                }
            }

            let deductions = XyzWing.execute(&grid).unwrap();
            assert!(
                deductions.is_empty(),
                "Expected no deductions when wings are far apart. Got: {deductions}"
            );
        }
    }

    strategy_snapshot_tests!(XyzWing);
}
