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
Y-Wing (XY-Wing) Strategy:

The Y-Wing technique involves three cells (all with exactly two candidates), forming a "Y" shape:
- The pivot cell has candidates (X, Y)
- Wing 1 shares a house (row, column, or box) with the pivot and has candidates (X, Z)
- Wing 2 shares a house (row, column, or box) with the pivot (but not necessarily with Wing 1) and has candidates (Y, Z)

The key insight is that no matter what value the pivot takes:
- If pivot = X, then Wing 1 = Z (since Wing 1 has candidates X, Z)
- If pivot = Y, then Wing 2 = Z (since Wing 2 has candidates Y, Z)

Therefore, at least one of the wings must be Z, so any cell that can see BOTH wings cannot contain Z.

Implementation approach:
1. Find all cells with exactly 2 candidates (bi-value cells)
2. For each potential pivot cell with candidates (X, Y):
   a. Find all cells that share a house with the pivot and have exactly 2 candidates
   b. Group these by which candidate they share with the pivot
   c. For Wing 1 candidates (X, Z): find cells with candidate X and one other (Z)
   d. For Wing 2 candidates (Y, Z): find cells with candidate Y and one other (Z)
   e. For each valid Wing 1 + Wing 2 pair that share Z:
      - Find cells that can see both wings and have Z as a candidate
      - Eliminate Z from those cells
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct YWing;

impl Strategy for YWing {
    fn name(self) -> &'static str {
        "YWing"
    }

    fn score(self) -> StrategyScore {
        // Y-Wing is typically scored between X-Wing (200) and more complex strategies
        // According to sudokuwiki.org, Y-Wing has a similar difficulty to X-Wing
        160
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(find_y_wings(grid))
    }
}

/// Find all Y-Wing patterns in the grid
fn find_y_wings<Base: SudokuBase>(grid: &Grid<Base>) -> Deductions<Base> {
    // Collect all bi-value cells (cells with exactly 2 candidates)
    let bi_value_cells: Vec<(Position<Base>, Candidates<Base>)> = Position::<Base>::all()
        .filter_map(|pos| {
            grid.get(pos)
                .candidates()
                .filter(|candidates| candidates.count() == 2)
                .map(|candidates| (pos, candidates))
        })
        .collect();

    // For each potential pivot, try to form Y-Wing patterns
    bi_value_cells
        .iter()
        .copied()
        .flat_map(|(pivot_pos, pivot_candidates)| {
            find_y_wings_for_pivot(grid, pivot_pos, pivot_candidates, &bi_value_cells)
        })
        .collect()
}

/// Find all Y-Wing patterns with a specific pivot
fn find_y_wings_for_pivot<Base: SudokuBase>(
    grid: &Grid<Base>,
    pivot_pos: Position<Base>,
    pivot_candidates: Candidates<Base>,
    bi_value_cells: &[(Position<Base>, Candidates<Base>)],
) -> Vec<Deduction<Base>> {
    // Get the two candidates of the pivot (X and Y)
    let mut pivot_iter = pivot_candidates.into_iter();
    let x = pivot_iter.next().unwrap();
    let y = pivot_iter.next().unwrap();

    // Find potential wing cells that share a house with the pivot
    let potential_wings: Vec<(Position<Base>, Candidates<Base>, Value<Base>)> = bi_value_cells
        .iter()
        .filter_map(|&(wing_pos, wing_candidates)| {
            // Wing must share a house with pivot but not be the same cell
            if wing_pos == pivot_pos || !shares_house(pivot_pos, wing_pos) {
                return None;
            }

            // Wing must share exactly one candidate with the pivot
            let shared_candidates = pivot_candidates.intersection(wing_candidates);
            if shared_candidates.count() != 1 {
                return None;
            }

            let shared = shared_candidates.into_iter().next().unwrap();
            Some((wing_pos, wing_candidates, shared))
        })
        .collect();

    // Find pairs of wings that form a valid Y-Wing
    // Wing 1: shares X with pivot, has candidates (X, Z)
    // Wing 2: shares Y with pivot, has candidates (Y, Z)
    let wing1_cells: Vec<_> = potential_wings
        .iter()
        .filter(|&&(_, _, shared)| shared == x)
        .map(|&(pos, candidates, _)| {
            let z = candidates.without(Candidates::with_single(x));
            (pos, candidates, z)
        })
        .collect();

    let wing2_cells: Vec<_> = potential_wings
        .iter()
        .filter(|&&(_, _, shared)| shared == y)
        .map(|&(pos, candidates, _)| {
            let z = candidates.without(Candidates::with_single(y));
            (pos, candidates, z)
        })
        .collect();

    // Generate deductions for all valid wing pairs
    wing1_cells
        .into_iter()
        .flat_map(|(wing1_pos, wing1_candidates, z1)| {
            let wing2_iter = wing2_cells.clone().into_iter();
            wing2_iter.filter_map(move |(wing2_pos, wing2_candidates, z2)| {
                // Wings must share the same Z candidate
                if z1 != z2 {
                    return None;
                }

                // Z is the common candidate in both wings that isn't in the pivot
                let z_candidate = z1.into_iter().next().unwrap();

                // Find cells that can see both wings and have Z as a candidate
                let positions_to_eliminate: Vec<Position<Base>> = Position::<Base>::all()
                    .filter(|&pos| {
                        // Must see both wings
                        pos != wing1_pos
                            && pos != wing2_pos
                            && pos != pivot_pos
                            && shares_house(pos, wing1_pos)
                            && shares_house(pos, wing2_pos)
                    })
                    .filter(|&pos| {
                        // Must have Z as a candidate
                        grid.get(pos)
                            .candidates()
                            .is_some_and(|candidates| candidates.has(z_candidate))
                    })
                    .collect();

                if positions_to_eliminate.is_empty() {
                    return None;
                }

                // Create deduction
                Some(
                    Deduction::try_from_iters(
                        positions_to_eliminate
                            .into_iter()
                            .map(|pos| (pos, Action::delete_candidate(z_candidate))),
                        [
                            (pivot_pos, Reason::candidates(pivot_candidates)),
                            (wing1_pos, Reason::candidates(wing1_candidates)),
                            (wing2_pos, Reason::candidates(wing2_candidates)),
                        ],
                    )
                    .unwrap(),
                )
            })
        })
        .collect()
}

/// Check if two positions share a house (row, column, or block)
fn shares_house<Base: SudokuBase>(pos1: Position<Base>, pos2: Position<Base>) -> bool {
    pos1.to_row() == pos2.to_row()
        || pos1.to_column() == pos2.to_column()
        || pos1.to_block() == pos2.to_block()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::consts::*, cell::Cell};
    use std::fmt::Debug;

    fn y_wing_deduction<Base: SudokuBase>(
        positions_to_delete: impl IntoIterator<Item = (u8, u8)>,
        delete_candidate: impl TryInto<Value<Base>, Error: Debug>,
        pivot: ((u8, u8), Vec<u8>),
        wing1: ((u8, u8), Vec<u8>),
        wing2: ((u8, u8), Vec<u8>),
    ) -> Deduction<Base> {
        let delete_candidate = delete_candidate.try_into().unwrap();
        Deduction::try_from_iters(
            positions_to_delete
                .into_iter()
                .map(|pos| (pos, Action::delete_candidate(delete_candidate))),
            [
                (
                    pivot.0,
                    Reason::candidates(Candidates::try_from(pivot.1).unwrap()),
                ),
                (
                    wing1.0,
                    Reason::candidates(Candidates::try_from(wing1.1).unwrap()),
                ),
                (
                    wing2.0,
                    Reason::candidates(Candidates::try_from(wing2.1).unwrap()),
                ),
            ],
        )
        .unwrap()
    }

    /// Create a grid with all cells having all candidates
    fn grid_with_all_candidates<Base: SudokuBase>() -> Grid<Base> {
        Grid::filled_with(Cell::with_candidates(Candidates::all()))
    }

    mod synthetic {
        use super::*;

        #[test]
        fn test_basic_y_wing() {
            // Create a synthetic grid for testing Y-Wing
            // Pivot at (0,0) with candidates [1,2]
            // Wing 1 at (0,1) with candidates [1,3] - shares row with pivot
            // Wing 2 at (1,0) with candidates [2,3] - shares column with pivot
            // Target at (1,1) can see both wings and has candidate 3

            type Base = Base2;
            let mut grid: Grid<Base> = grid_with_all_candidates();

            // Set up pivot (0,0) with candidates [1,2]
            grid[(0u8, 0u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());

            // Set up Wing 1 (0,1) with candidates [1,3]
            grid[(0u8, 1u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 3]).unwrap());

            // Set up Wing 2 (1,0) with candidates [2,3]
            grid[(1u8, 0u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![2, 3]).unwrap());

            // Set up target (1,1) with candidates including 3
            grid[(1u8, 1u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![3, 4]).unwrap());

            let deductions = YWing.execute(&grid).unwrap();

            // The deduction should eliminate candidate 3 from position (1,1)
            assert!(!deductions.is_empty(), "Expected at least one deduction");

            // Check that one of the deductions matches our expected Y-Wing pattern
            let expected_deduction = y_wing_deduction(
                vec![(1, 1)],
                3,
                ((0, 0), vec![1, 2]),
                ((0, 1), vec![1, 3]),
                ((1, 0), vec![2, 3]),
            );

            assert!(
                deductions.iter().any(|d| *d == expected_deduction),
                "Expected deduction not found in {:?}",
                deductions
            );
        }

        #[test]
        fn test_no_y_wing_when_no_common_z() {
            // Pivot at (0,0) with candidates [1,2]
            // Wing 1 at (0,1) with candidates [1,3]
            // Wing 2 at (1,0) with candidates [2,4] - different Z!
            // No Y-Wing should be found because Z values don't match

            type Base = Base2;
            let mut grid: Grid<Base> = grid_with_all_candidates();

            grid[(0u8, 0u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());
            grid[(0u8, 1u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 3]).unwrap());
            grid[(1u8, 0u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![2, 4]).unwrap());

            let deductions = YWing.execute(&grid).unwrap();
            assert!(
                deductions.is_empty(),
                "Expected no deductions when Z values don't match"
            );
        }

        #[test]
        fn test_no_y_wing_when_no_target() {
            // Test that Y-Wing doesn't produce deduction when target cell lacks the Z candidate
            // We need to carefully construct a grid where:
            // 1. There's a valid Y-Wing pattern (pivot, wing1, wing2)
            // 2. But no cell can see both wings that has Z as candidate

            type Base = Base2;
            // Create grid where most cells are not bi-value (so they won't form other Y-Wings)
            let mut grid: Grid<Base> = Grid::filled_with(Cell::with_candidates(Candidates::all()));

            let pivot_pos: Position<Base> = (0u8, 0u8).try_into().unwrap();
            let wing1_pos: Position<Base> = (0u8, 3u8).try_into().unwrap(); // Same row, different block
            let wing2_pos: Position<Base> = (3u8, 0u8).try_into().unwrap(); // Same column, different block

            // Pivot has candidates [1,2]
            grid[pivot_pos] = Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());
            // Wing1 shares candidate 1 with pivot, has Z=3
            grid[wing1_pos] = Cell::with_candidates(Candidates::try_from(vec![1, 3]).unwrap());
            // Wing2 shares candidate 2 with pivot, has Z=3
            grid[wing2_pos] = Cell::with_candidates(Candidates::try_from(vec![2, 3]).unwrap());

            // The only cell that can see both wings is (3,3)
            // Remove candidate 3 from it so no Y-Wing deduction is possible
            let target_pos: Position<Base> = (3u8, 3u8).try_into().unwrap();
            grid[target_pos] = Cell::with_candidates(Candidates::try_from(vec![1, 2, 4]).unwrap());

            let deductions = YWing.execute(&grid).unwrap();

            // Filter to find deductions for our specific Y-Wing pattern
            let relevant_deductions: Vec<_> = deductions
                .iter()
                .filter(|d| {
                    d.reasons.iter().any(|(pos, _)| pos == pivot_pos)
                        && d.reasons.iter().any(|(pos, _)| pos == wing1_pos)
                        && d.reasons.iter().any(|(pos, _)| pos == wing2_pos)
                })
                .collect();

            assert!(
                relevant_deductions.is_empty(),
                "Expected no deductions for this specific Y-Wing pattern. Found: {:?}",
                relevant_deductions
            );
        }
    }

    mod base_3_tests {
        use super::*;

        #[test]
        fn test_y_wing_across_block() {
            // Test Y-Wing where wings are in different blocks but share visibility
            // This tests a more realistic scenario on a 9x9 grid

            type Base = Base3;
            let mut grid: Grid<Base> = grid_with_all_candidates();

            // Set up a Y-Wing pattern:
            // Pivot at (0,0) with candidates [1,2]
            // Wing 1 at (0,8) with candidates [1,9] - same row as pivot
            // Wing 2 at (8,0) with candidates [2,9] - same column as pivot
            // Target at (8,8) which can see both wings (same row as wing2, same column as wing1)

            grid[(0u8, 0u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());
            grid[(0u8, 8u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 9]).unwrap());
            grid[(8u8, 0u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![2, 9]).unwrap());
            grid[(8u8, 8u8).try_into().unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![5, 9]).unwrap());

            let deductions = YWing.execute(&grid).unwrap();

            // Should find the Y-Wing and eliminate 9 from (8,8)
            assert!(!deductions.is_empty(), "Expected at least one deduction");

            let expected_deduction = y_wing_deduction(
                vec![(8, 8)],
                9,
                ((0, 0), vec![1, 2]),
                ((0, 8), vec![1, 9]),
                ((8, 0), vec![2, 9]),
            );

            assert!(
                deductions.iter().any(|d| *d == expected_deduction),
                "Expected Y-Wing deduction not found. Found: {:?}",
                deductions
            );
        }
    }
}
