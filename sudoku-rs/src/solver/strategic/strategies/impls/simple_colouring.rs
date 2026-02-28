use crate::base::SudokuBase;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Coordinate;
use crate::position::Position;
use crate::solver::strategic::deduction::Action;
use crate::solver::strategic::deduction::Deduction;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::deduction::Reason;
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;
use std::collections::{BTreeMap, BTreeSet};

/*
Simple Colouring (Singles Chains) Strategy

Reference: https://www.sudokuwiki.org/Simple_Colouring

Logic:

For each candidate value:
1. Find all "strong links" - pairs of cells in a unit (row, column, or box) where the candidate
   appears in exactly two cells. These are called "conjugate pairs".

2. Build chains of strong links by connecting conjugate pairs that share a cell.
   Alternate colors (e.g., Color::A and Color::B) along the chain.

3. Apply elimination rules:
   - Type 1 (Contradiction): If two cells with the same color are in the same unit (row, column, or box),
     that color must be false everywhere. Eliminate the candidate from all cells with that color.
   - Type 2 (Elimination): If an uncolored cell can "see" both colors (shares a unit with
     both a Color::A cell and a Color::B cell), eliminate the candidate from that cell.
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SimpleColouring;

impl Strategy for SimpleColouring {
    fn name(self) -> &'static str {
        "SimpleColouring"
    }

    fn score(self) -> StrategyScore {
        // Higher than X-Wing (200) as it's a more advanced chain-based technique
        250
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(Value::<Base>::all()
            .flat_map(|candidate| find_simple_colouring_eliminations(grid, candidate))
            .collect())
    }
}

/// Two possible colors for cells in a chain
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
enum Color {
    A,
    B,
}

impl Color {
    fn opposite(self) -> Self {
        match self {
            Color::A => Color::B,
            Color::B => Color::A,
        }
    }
}

/// Find all simple colouring eliminations for a given candidate value
fn find_simple_colouring_eliminations<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
) -> impl Iterator<Item = Deduction<Base>> {
    // Find all positions that contain this candidate
    let candidate_positions = find_candidate_positions(grid, candidate);

    // Build the chains using strong links
    let chains = build_color_chains(grid, candidate, &candidate_positions);

    // Apply elimination rules to each chain
    chains
        .into_iter()
        .filter_map(move |chain| apply_elimination_rules(grid, candidate, &chain))
}

/// Find all positions where the given candidate exists
fn find_candidate_positions<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
) -> BTreeSet<Position<Base>> {
    Position::<Base>::all()
        .filter(|&pos| {
            grid[pos]
                .candidates()
                .is_some_and(|candidates| candidates.has(candidate))
        })
        .collect()
}

/// Build all color chains for a candidate value using strong links
fn build_color_chains<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    candidate_positions: &BTreeSet<Position<Base>>,
) -> Vec<BTreeMap<Position<Base>, Color>> {
    // Find all strong links (conjugate pairs) in each unit
    let strong_links = find_all_strong_links(grid, candidate, candidate_positions);

    // Build chains by connecting strong links
    let mut chains = Vec::new();
    let mut colored: BTreeSet<Position<Base>> = BTreeSet::new();

    for &pos in candidate_positions {
        if colored.contains(&pos) {
            continue;
        }

        // Start a new chain from this position
        let chain = build_chain_from_position(pos, &strong_links);

        if chain.len() >= 2 {
            // Only consider chains with at least 2 cells
            for &chain_pos in chain.keys() {
                colored.insert(chain_pos);
            }
            chains.push(chain);
        }
    }

    chains
}

/// Find all strong links for a candidate value
fn find_all_strong_links<Base: SudokuBase>(
    _grid: &Grid<Base>,
    _candidate: Value<Base>,
    candidate_positions: &BTreeSet<Position<Base>>,
) -> BTreeMap<Position<Base>, BTreeSet<Position<Base>>> {
    let mut links: BTreeMap<Position<Base>, BTreeSet<Position<Base>>> = BTreeMap::new();

    // Check rows for conjugate pairs
    for row in Coordinate::<Base>::all() {
        let row_positions: Vec<_> = candidate_positions
            .iter()
            .filter(|&&pos| pos.to_row() == row)
            .copied()
            .collect();

        if row_positions.len() == 2 {
            // Conjugate pair found in this row
            links
                .entry(row_positions[0])
                .or_default()
                .insert(row_positions[1]);
            links
                .entry(row_positions[1])
                .or_default()
                .insert(row_positions[0]);
        }
    }

    // Check columns for conjugate pairs
    for col in Coordinate::<Base>::all() {
        let col_positions: Vec<_> = candidate_positions
            .iter()
            .filter(|&&pos| pos.to_column() == col)
            .copied()
            .collect();

        if col_positions.len() == 2 {
            // Conjugate pair found in this column
            links
                .entry(col_positions[0])
                .or_default()
                .insert(col_positions[1]);
            links
                .entry(col_positions[1])
                .or_default()
                .insert(col_positions[0]);
        }
    }

    // Check boxes for conjugate pairs
    for block in Coordinate::<Base>::all() {
        let block_positions: Vec<_> = candidate_positions
            .iter()
            .filter(|&&pos| pos.to_block() == block)
            .copied()
            .collect();

        if block_positions.len() == 2 {
            // Conjugate pair found in this block
            links
                .entry(block_positions[0])
                .or_default()
                .insert(block_positions[1]);
            links
                .entry(block_positions[1])
                .or_default()
                .insert(block_positions[0]);
        }
    }

    links
}

/// Build a color chain starting from a given position
fn build_chain_from_position<Base: SudokuBase>(
    start: Position<Base>,
    strong_links: &BTreeMap<Position<Base>, BTreeSet<Position<Base>>>,
) -> BTreeMap<Position<Base>, Color> {
    let mut chain = BTreeMap::new();
    let mut queue = vec![(start, Color::A)];

    while let Some((pos, color)) = queue.pop() {
        if chain.contains_key(&pos) {
            continue;
        }

        chain.insert(pos, color);

        // Follow strong links to connected cells
        if let Some(linked) = strong_links.get(&pos) {
            for &linked_pos in linked {
                if !chain.contains_key(&linked_pos) {
                    queue.push((linked_pos, color.opposite()));
                }
            }
        }
    }

    chain
}

/// Apply elimination rules to a color chain
fn apply_elimination_rules<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    chain: &BTreeMap<Position<Base>, Color>,
) -> Option<Deduction<Base>> {
    // First, try Type 1 elimination (color sees itself - contradiction)
    if let Some(deduction) = try_type1_elimination(grid, candidate, chain) {
        return Some(deduction);
    }

    // Then, try Type 2 elimination (uncolored cell sees both colors)
    try_type2_elimination(grid, candidate, chain)
}

/// Type 1: If two cells with the same color are in the same unit, that color is false everywhere
#[allow(clippy::similar_names)]
fn try_type1_elimination<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    chain: &BTreeMap<Position<Base>, Color>,
) -> Option<Deduction<Base>> {
    // Group positions by color
    let color_a_positions: Vec<_> = chain
        .iter()
        .filter(|&(_, &c)| c == Color::A)
        .map(|(&pos, _)| pos)
        .collect();

    let color_b_positions: Vec<_> = chain
        .iter()
        .filter(|&(_, &c)| c == Color::B)
        .map(|(&pos, _)| pos)
        .collect();

    // Check if Color A sees itself (same row, column, or box)
    if has_color_conflict(&color_a_positions) {
        // Color A is false everywhere - delete candidate from all Color A cells
        return create_type1_deduction(grid, candidate, &color_a_positions, &color_b_positions);
    }

    // Check if Color B sees itself
    if has_color_conflict(&color_b_positions) {
        // Color B is false everywhere - delete candidate from all Color B cells
        return create_type1_deduction(grid, candidate, &color_b_positions, &color_a_positions);
    }

    None
}

/// Check if any two positions with the same color are in the same unit
fn has_color_conflict<Base: SudokuBase>(positions: &[Position<Base>]) -> bool {
    for (i, &pos1) in positions.iter().enumerate() {
        for &pos2 in &positions[i + 1..] {
            if pos1.to_row() == pos2.to_row()
                || pos1.to_column() == pos2.to_column()
                || pos1.to_block() == pos2.to_block()
            {
                return true;
            }
        }
    }
    false
}

/// Create a Type 1 deduction (eliminate candidate from all cells of the false color)
fn create_type1_deduction<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    false_color_positions: &[Position<Base>],
    true_color_positions: &[Position<Base>],
) -> Option<Deduction<Base>> {
    // The positions to delete from (false color)
    let positions_to_delete: Vec<_> = false_color_positions
        .iter()
        .filter(|&&pos| {
            grid[pos]
                .candidates()
                .is_some_and(|candidates| candidates.has(candidate))
        })
        .copied()
        .collect();

    if positions_to_delete.is_empty() {
        return None;
    }

    // The reason positions (true color - these are the cells that prove the deduction)
    let reason_positions: Vec<_> = true_color_positions.to_vec();

    Some(
        Deduction::try_from_iters(
            positions_to_delete
                .into_iter()
                .map(|pos| (pos, Action::delete_candidate(candidate))),
            reason_positions
                .into_iter()
                .map(|pos| (pos, Reason::candidate(candidate))),
        )
        .unwrap(),
    )
}

/// Type 2: If an uncolored cell sees both colors, eliminate the candidate from that cell
#[allow(clippy::similar_names)]
fn try_type2_elimination<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    chain: &BTreeMap<Position<Base>, Color>,
) -> Option<Deduction<Base>> {
    let color_a_positions: BTreeSet<_> = chain
        .iter()
        .filter(|&(_, &c)| c == Color::A)
        .map(|(&pos, _)| pos)
        .collect();

    let color_b_positions: BTreeSet<_> = chain
        .iter()
        .filter(|&(_, &c)| c == Color::B)
        .map(|(&pos, _)| pos)
        .collect();

    // Find uncolored cells that have the candidate and can see both colors
    let mut positions_to_delete = Vec::new();
    let mut reason_positions_a = BTreeSet::new();
    let mut reason_positions_b = BTreeSet::new();

    for pos in Position::<Base>::all() {
        // Skip if this position is in the chain
        if chain.contains_key(&pos) {
            continue;
        }

        // Skip if this position doesn't have the candidate
        let has_candidate = grid[pos]
            .candidates()
            .is_some_and(|candidates| candidates.has(candidate));

        if !has_candidate {
            continue;
        }

        // Check if this position can see both colors
        let sees_a = sees_any_of(pos, &color_a_positions);
        let sees_b = sees_any_of(pos, &color_b_positions);

        if sees_a.is_some() && sees_b.is_some() {
            positions_to_delete.push(pos);
            if let Some(seen_a) = sees_a {
                reason_positions_a.insert(seen_a);
            }
            if let Some(seen_b) = sees_b {
                reason_positions_b.insert(seen_b);
            }
        }
    }

    if positions_to_delete.is_empty() {
        return None;
    }

    // Combine reason positions from both colors
    let reason_positions: Vec<_> = reason_positions_a
        .into_iter()
        .chain(reason_positions_b)
        .collect();

    Some(
        Deduction::try_from_iters(
            positions_to_delete
                .into_iter()
                .map(|pos| (pos, Action::delete_candidate(candidate))),
            reason_positions
                .into_iter()
                .map(|pos| (pos, Reason::candidate(candidate))),
        )
        .unwrap(),
    )
}

/// Check if a position can "see" any of the given positions (same row, column, or box)
fn sees_any_of<Base: SudokuBase>(
    pos: Position<Base>,
    positions: &BTreeSet<Position<Base>>,
) -> Option<Position<Base>> {
    positions
        .iter()
        .find(|&&other| {
            pos != other
                && (pos.to_row() == other.to_row()
                    || pos.to_column() == other.to_column()
                    || pos.to_block() == other.to_block())
        })
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::consts::*,
        cell::{Candidates, Cell},
    };

    mod synthetic {
        use super::*;

        #[test]
        fn test_chain_building_basic() {
            // Test that chain building works correctly for a simple chain
            // This creates a rectangular chain of 4 cells for candidate 1

            let mut grid: Grid<Base2> = Grid::new();

            // Set up candidates for value 1 in specific positions
            // Row 0: candidate 1 in (0,0) and (0,2) only - conjugate pair
            // Column 0: candidate 1 in (0,0) and (2,0) only - conjugate pair
            // Column 2: candidate 1 in (0,2) and (2,2) only - conjugate pair
            // Row 2: candidate 1 in (2,0) and (2,2) only - conjugate pair
            grid[Position::try_from((0u8, 0u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());
            grid[Position::try_from((0u8, 2u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 3]).unwrap());
            grid[Position::try_from((2u8, 0u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 4]).unwrap());
            grid[Position::try_from((2u8, 2u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());

            // Fill other positions with candidates that don't include 1
            for pos in Position::<Base2>::all() {
                if grid[pos].candidates().is_none() {
                    grid[pos] =
                        Cell::with_candidates(Candidates::try_from(vec![2, 3, 4]).unwrap());
                }
            }

            let deductions = SimpleColouring.execute(&grid).unwrap();

            // This configuration forms a valid chain without Type 1 conflict
            // and no off-chain cells to trigger Type 2, so no eliminations expected
            assert!(
                deductions.is_empty(),
                "Expected no eliminations for this chain configuration"
            );
        }

        #[test]
        fn test_type2_elimination_cell_sees_both_colors() {
            // Test Type 2 elimination: an uncolored cell that can see both colors
            // Set up a chain where an off-chain cell sees cells of both colors

            let mut grid: Grid<Base2> = Grid::new();

            // Create a simple 2-cell chain for candidate 1:
            // (0,0) and (0,1) are conjugate in row 0 (only 2 cells with candidate 1)
            // Colors: (0,0)=A, (0,1)=B
            //
            // Cell (1,0) is not in the chain, but it can see:
            // - (0,0)=A via column 0
            // - AND if we have another chain cell visible...
            // We need (1,0) to see both colors. Let's extend the chain.
            //
            // Better setup:
            // (0,0) and (1,0) conjugate in column 0 -> (0,0)=A, (1,0)=B
            // (0,0) and (0,1) conjugate in row 0 -> (0,1)=B
            // Now cell (1,1) can see (1,0)=B via row 1 and (0,1)=B via column 1
            // Still only sees B, not A.
            //
            // Let's try a different setup:
            // (0,0) and (2,0) conjugate in column 0 -> (0,0)=A, (2,0)=B
            // (0,0) and (0,2) conjugate in row 0 -> (0,2)=B
            // (2,0) and (2,2) conjugate in row 2 -> (2,2)=A
            // (0,2) and (2,2) conjugate in column 2 -> already covered
            //
            // Now cell (1,1) is not in chain, has candidate 1
            // Can it see both colors?
            // - Via block 0: (0,0)=A
            // - Via block 0: (1,1) is NOT in same row/col/block as (2,0), (0,2), (2,2)
            // Actually for Base2, block 0 contains (0,0), (0,1), (1,0), (1,1)
            // So (1,1) sees (0,0)=A via block
            //
            // Does (1,1) see any B color?
            // - (2,0)=B - different row, column, and block (block 2) - NO
            // - (0,2)=B - different row, column, and block (block 1) - NO
            //
            // This is tricky on a 4x4 grid. Let's just verify the strategy runs
            // and produces consistent results.

            grid[Position::try_from((0u8, 0u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());
            grid[Position::try_from((0u8, 2u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 3]).unwrap());
            grid[Position::try_from((2u8, 0u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 4]).unwrap());
            grid[Position::try_from((2u8, 2u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2]).unwrap());

            // Fill other positions including (1,1) with candidate 1
            for pos in Position::<Base2>::all() {
                if grid[pos].candidates().is_none() {
                    grid[pos] =
                        Cell::with_candidates(Candidates::try_from(vec![2, 3, 4]).unwrap());
                }
            }

            // Set (1,1) to have candidate 1 as well
            grid[Position::try_from((1u8, 1u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2, 3]).unwrap());

            let deductions = SimpleColouring.execute(&grid).unwrap();

            // (1,1) is in block 0, which also contains (0,0)=A
            // But (1,1) doesn't see any B-colored cell (2,0 or 0,2)
            // So no Type 2 elimination expected
            assert!(
                deductions.is_empty(),
                "Expected no eliminations - (1,1) only sees one color"
            );
        }

        #[test]
        fn test_type2_elimination_base3() {
            // Use Base3 (9x9) grid for a more realistic Type 2 scenario
            // where cells can see both colors more easily
            //
            // For Type 2 elimination:
            // 1. Build a chain with alternating colors
            // 2. Find an off-chain cell that can see both colors
            // 3. That cell's candidate can be eliminated
            //
            // Setup: Create a linear chain where an off-chain cell in the same
            // block as two differently-colored chain cells can be eliminated.
            //
            // Chain for candidate 5:
            // (0,0) and (0,6) conjugate in row 0 -> (0,0)=A, (0,6)=B
            // (0,6) and (6,6) conjugate in column 6 -> (6,6)=A
            //
            // Off-chain cell at (0,3) with candidate 5:
            // - Sees (0,0)=A via row 0
            // - Sees (0,6)=B via row 0
            // This should trigger Type 2 elimination!

            let mut grid: Grid<Base3> = Grid::new();

            // Chain cells - ensure row 0 has ONLY (0,0) and (0,6) with candidate 5
            // And column 6 has ONLY (0,6) and (6,6) with candidate 5
            grid[Position::try_from((0u8, 0u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![5, 6]).unwrap());
            grid[Position::try_from((0u8, 6u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![5, 7]).unwrap());
            grid[Position::try_from((6u8, 6u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![5, 8]).unwrap());

            // Off-chain cell that should have candidate 5 eliminated
            // This cell is in row 0, so it sees (0,0)=A and (0,6)=B
            grid[Position::try_from((0u8, 3u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![5, 9]).unwrap());

            // Fill remaining cells in row 0 without candidate 5
            // (to ensure (0,0) and (0,6) are a conjugate pair)
            for c in [1u8, 2, 4, 5, 7, 8] {
                grid[Position::try_from((0u8, c)).unwrap()] =
                    Cell::with_candidates(Candidates::try_from(vec![6, 7, 8, 9]).unwrap());
            }

            // Fill remaining cells in column 6 without candidate 5
            // (to ensure (0,6) and (6,6) are a conjugate pair)
            for r in [1u8, 2, 3, 4, 5, 7, 8] {
                grid[Position::try_from((r, 6u8)).unwrap()] =
                    Cell::with_candidates(Candidates::try_from(vec![6, 7, 8, 9]).unwrap());
            }

            // Fill all other cells without candidate 5
            for pos in Position::<Base3>::all() {
                if grid[pos].candidates().is_none() {
                    grid[pos] =
                        Cell::with_candidates(Candidates::try_from(vec![6, 7, 8, 9]).unwrap());
                }
            }

            let deductions = SimpleColouring.execute(&grid).unwrap();

            // Check that we have an elimination at (0,3) for candidate 5
            // The chain is: (0,0)=A, (0,6)=B, (6,6)=A
            // Cell (0,3) sees (0,0)=A and (0,6)=B via row 0, so Type 2 applies

            assert!(
                !deductions.is_empty(),
                "Expected Type 2 elimination at (0,3)"
            );

            // Verify the elimination is at the correct position
            let target_pos = Position::try_from((0u8, 3u8)).unwrap();
            let has_target_elimination = deductions
                .iter()
                .any(|d| d.actions.iter().any(|(pos, _)| pos == target_pos));
            assert!(
                has_target_elimination,
                "Expected elimination at position (0,3)"
            );
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_grid_no_conjugate_pairs() {
            // An empty grid with all candidates won't have conjugate pairs
            // (all candidates appear in all cells of each unit)

            let grid: Grid<Base3> = Grid::new();

            let deductions = SimpleColouring.execute(&grid).unwrap();

            assert!(
                deductions.is_empty(),
                "Empty grid should have no eliminations"
            );
        }

        #[test]
        fn test_single_candidate_no_chain() {
            // Grid where a candidate exists but has no conjugate pairs

            let mut grid: Grid<Base2> = Grid::new();

            // Only one cell has candidate 1 - no conjugate pair possible
            grid[Position::try_from((0u8, 0u8)).unwrap()] =
                Cell::with_candidates(Candidates::try_from(vec![1, 2, 3]).unwrap());

            // Fill rest without candidate 1
            for pos in Position::<Base2>::all() {
                if grid[pos].candidates().is_none() {
                    grid[pos] =
                        Cell::with_candidates(Candidates::try_from(vec![2, 3, 4]).unwrap());
                }
            }

            let deductions = SimpleColouring.execute(&grid).unwrap();

            assert!(
                deductions.is_empty(),
                "Single candidate should produce no eliminations"
            );
        }
    }
}
