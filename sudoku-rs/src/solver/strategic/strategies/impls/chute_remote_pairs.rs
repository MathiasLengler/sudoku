use std::collections::BTreeMap;

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};
use crate::solver::strategic::strategies::{Strategy, StrategyScore};

/// Remote Pairs strategy.
///
/// Finds chains of bivalent cells (cells with exactly 2 candidates) that all share the
/// same pair of candidates, where each consecutive pair in the chain shares a unit
/// (row, column, or block).
///
/// The algorithm uses bipartite graph coloring: in a valid remote pair chain, the cells
/// alternate between two values. If the graph of same-pair bivalent cells is bipartite,
/// we can 2-color it. Any non-chain cell that can see at least one cell of each color
/// cannot contain either candidate, since one color must hold one value and the other
/// color must hold the other.
///
/// Requires connected components of at least 4 cells to avoid overlap with `NakedPairs`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChuteRemotePairs;

impl Strategy for ChuteRemotePairs {
    fn name(self) -> &'static str {
        "ChuteRemotePairs"
    }

    fn score(self) -> StrategyScore {
        150
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        let bivalve_cells = find_bivalve_cells(grid);
        let grouped = group_by_candidates(&bivalve_cells);

        let deductions: Deductions<Base> = grouped
            .into_iter()
            .flat_map(|(pair_candidates, positions)| {
                find_remote_pair_eliminations(grid, pair_candidates, &positions)
            })
            .collect();

        deductions.merge_deductions_by_reasons()
    }
}

/// Find all bivalent cells (cells with exactly 2 candidates).
fn find_bivalve_cells<Base: SudokuBase>(
    grid: &Grid<Base>,
) -> Vec<(Position<Base>, Candidates<Base>)> {
    grid.all_candidates_positions()
        .into_iter()
        .filter_map(|pos| {
            let candidates = grid.get(pos).candidates().unwrap();
            (candidates.count() == 2).then_some((pos, candidates))
        })
        .collect()
}

/// Group bivalent cells by their candidate pair.
fn group_by_candidates<Base: SudokuBase>(
    bivalve_cells: &[(Position<Base>, Candidates<Base>)],
) -> BTreeMap<Candidates<Base>, Vec<Position<Base>>> {
    let mut groups: BTreeMap<Candidates<Base>, Vec<Position<Base>>> = BTreeMap::new();
    for &(pos, candidates) in bivalve_cells {
        groups
            .entry(candidates)
            .and_modify(|positions| positions.push(pos))
            .or_insert_with(|| vec![pos]);
    }
    groups
}

/// Check if two positions share a unit (row, column, or block).
fn positions_see_each_other<Base: SudokuBase>(a: Position<Base>, b: Position<Base>) -> bool {
    a.to_row() == b.to_row() || a.to_column() == b.to_column() || a.to_block() == b.to_block()
}

/// Build adjacency list for positions that see each other.
fn build_adjacency<Base: SudokuBase>(positions: &[Position<Base>]) -> Vec<Vec<usize>> {
    positions
        .iter()
        .enumerate()
        .map(|(i, &pos_i)| {
            positions
                .iter()
                .enumerate()
                .filter(|&(j, &pos_j)| i != j && positions_see_each_other::<Base>(pos_i, pos_j))
                .map(|(j, _)| j)
                .collect()
        })
        .collect()
}

/// Try to 2-color a connected component using BFS, returning the coloring if bipartite.
/// Returns false if the component is not bipartite.
fn bipartite_coloring(
    start: usize,
    adjacency: &[Vec<usize>],
    color: &mut [Option<u8>],
) -> bool {
    use std::collections::VecDeque;
    let mut queue = VecDeque::new();
    color[start] = Some(0);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        let current_color = color[current].unwrap();
        let neighbor_color = 1 - current_color;

        for &neighbor in &adjacency[current] {
            match color[neighbor] {
                None => {
                    color[neighbor] = Some(neighbor_color);
                    queue.push_back(neighbor);
                }
                Some(c) if c != neighbor_color => {
                    return false; // Not bipartite (odd cycle)
                }
                _ => {} // Already colored correctly
            }
        }
    }
    true
}

/// Collect all members of a connected component via BFS, marking them as visited.
fn collect_component(
    start: usize,
    adjacency: &[Vec<usize>],
    visited: &mut [bool],
) -> Vec<usize> {
    use std::collections::VecDeque;
    let mut component = Vec::new();
    let mut queue = VecDeque::new();
    visited[start] = true;
    queue.push_back(start);
    component.push(start);

    while let Some(current) = queue.pop_front() {
        for &neighbor in &adjacency[current] {
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back(neighbor);
                component.push(neighbor);
            }
        }
    }
    component
}

/// For a group of bivalent cells with the same candidate pair, find all remote pair
/// eliminations using bipartite graph coloring.
fn find_remote_pair_eliminations<Base: SudokuBase>(
    grid: &Grid<Base>,
    pair_candidates: Candidates<Base>,
    positions: &[Position<Base>],
) -> Vec<Deduction<Base>> {
    if positions.len() < 4 {
        return Vec::new();
    }

    let adjacency = build_adjacency::<Base>(positions);

    let mut visited = vec![false; positions.len()];
    let mut color: Vec<Option<u8>> = vec![None; positions.len()];
    let mut deductions = Vec::new();

    for start in 0..positions.len() {
        if visited[start] {
            continue;
        }

        // Collect component members
        let component = collect_component(start, &adjacency, &mut visited);

        // Skip small components (naked pairs already handles size ≤ 3)
        if component.len() < 4 {
            continue;
        }

        // Try bipartite coloring
        if !bipartite_coloring(start, &adjacency, &mut color) {
            continue;
        }

        // Split component by color
        let mut color_0: Vec<Position<Base>> = Vec::new();
        let mut color_1: Vec<Position<Base>> = Vec::new();
        for &idx in &component {
            match color[idx] {
                Some(0) => color_0.push(positions[idx]),
                Some(1) => color_1.push(positions[idx]),
                _ => unreachable!(),
            }
        }

        // Find eliminations: any non-chain cell that sees at least one cell from each
        // color group can have both candidates eliminated
        let mut deduction = Deduction::new();

        for pos in Position::<Base>::all() {
            // Skip chain cells
            if color_0.contains(&pos) || color_1.contains(&pos) {
                continue;
            }

            // Check if this cell has any of the pair candidates
            let Some(cell_candidates) = grid.get(pos).candidates() else {
                continue;
            };
            let to_delete = cell_candidates.intersection(pair_candidates);
            if to_delete.is_empty() {
                continue;
            }

            // Check if this cell sees at least one cell from each color group
            let sees_color_0 = color_0
                .iter()
                .any(|&chain_pos| positions_see_each_other::<Base>(pos, chain_pos));
            let sees_color_1 = color_1
                .iter()
                .any(|&chain_pos| positions_see_each_other::<Base>(pos, chain_pos));

            if sees_color_0 && sees_color_1 {
                deduction
                    .actions
                    .insert(pos, Action::DeleteCandidates(to_delete))
                    .unwrap();
            }
        }

        if !deduction.actions.is_empty() {
            // Add all chain cells as reasons
            for &chain_pos in color_0.iter().chain(color_1.iter()) {
                deduction
                    .reasons
                    .insert(chain_pos, Reason::Candidates(pair_candidates))
                    .unwrap();
            }
            deductions.push(deduction);
        }
    }

    deductions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::consts::*,
        cell::Cell,
        solver::strategic::strategies::test_util::strategy_snapshot_tests,
    };

    mod synthetic {
        use super::*;

        /// Test with a Base3 grid containing a 4-cell remote pair chain.
        ///
        /// Chain for candidates {3,7}:
        /// (0,0) {3,7} → (0,3) {3,7} [same row]
        ///                    ↓
        /// (3,6) {3,7} ← (3,3) {3,7} [same column]
        ///
        /// Bipartite coloring:
        /// Color 0: {(0,0), (3,3)}
        /// Color 1: {(0,3), (3,6)}
        ///
        /// Cell (0,6) sees (0,0) [row, color 0] and (3,6) [column, color 1]
        /// → eliminate 3 and 7 from (0,6)
        #[test]
        fn test_synthetic_base_3_path_chain() {
            let mut grid = Grid::<Base3>::default();

            // Set up chain cells with {3,7}
            let chain_candidates: Candidates<Base3> =
                Candidates::try_from(vec![3, 7]).unwrap();

            let chain_positions: Vec<Position<Base3>> = vec![
                (0u8, 0u8).try_into().unwrap(),
                (0, 3).try_into().unwrap(),
                (3, 3).try_into().unwrap(),
                (3, 6).try_into().unwrap(),
            ];

            for &pos in &chain_positions {
                grid[pos] = Cell::with_candidates(chain_candidates);
            }

            // Set up a cell that sees one from each color group
            // (0,6) is in row 0 (sees (0,0) color 0) and column 6 (sees (3,6) color 1)
            let target_pos: Position<Base3> = (0u8, 6u8).try_into().unwrap();
            let target_candidates: Candidates<Base3> =
                Candidates::try_from(vec![1, 3, 5, 7]).unwrap();
            grid[target_pos] = Cell::with_candidates(target_candidates);

            // Set remaining cells to have candidates that don't interfere
            for pos in Position::<Base3>::all() {
                if grid.get(pos).candidates().is_none() && !grid.get(pos).has_value() {
                    let other_candidates: Candidates<Base3> =
                        Candidates::try_from(vec![1, 2, 4, 5, 6, 8, 9]).unwrap();
                    grid[pos] = Cell::with_candidates(other_candidates);
                }
            }

            let deductions = ChuteRemotePairs.execute(&grid).unwrap();

            assert!(
                !deductions.is_empty(),
                "Expected at least one deduction from remote pairs"
            );

            // Verify the deduction eliminates {3,7} from (0,6)
            let expected_delete_candidates: Candidates<Base3> =
                Candidates::try_from(vec![3, 7]).unwrap();

            let deduction = deductions.iter().next().unwrap();

            let action = deduction.actions.iter()
                .find(|(pos, _)| *pos == target_pos);
            assert!(action.is_some(), "Expected elimination at (0,6)");
            let (_, action) = action.unwrap();
            assert_eq!(
                *action,
                Action::DeleteCandidates(expected_delete_candidates)
            );
        }

        /// Test that no deductions are found when the component is too small (< 4 cells).
        #[test]
        fn test_no_deduction_for_small_component() {
            let mut grid = Grid::<Base3>::default();

            // Only 2 bivalent cells with same pair - too small for remote pairs
            let chain_candidates: Candidates<Base3> =
                Candidates::try_from(vec![3, 7]).unwrap();

            let pos1: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            let pos2: Position<Base3> = (0, 3).try_into().unwrap();
            grid[pos1] = Cell::with_candidates(chain_candidates);
            grid[pos2] = Cell::with_candidates(chain_candidates);

            // Fill rest with non-interfering candidates
            for pos in Position::<Base3>::all() {
                if grid.get(pos).candidates().is_none() && !grid.get(pos).has_value() {
                    let other_candidates: Candidates<Base3> =
                        Candidates::try_from(vec![1, 2, 4, 5, 6, 8, 9]).unwrap();
                    grid[pos] = Cell::with_candidates(other_candidates);
                }
            }

            let deductions = ChuteRemotePairs.execute(&grid).unwrap();
            assert!(
                deductions.is_empty(),
                "Should not find remote pairs with only 2 bivalent cells"
            );
        }

        /// Test that non-bipartite components (odd cycles) produce no deductions.
        #[test]
        fn test_no_deduction_for_odd_cycle() {
            let mut grid = Grid::<Base3>::default();

            // 3 bivalent cells forming a triangle (odd cycle) - not bipartite
            // (0,0) sees (0,1) [same row, same block]
            // (0,1) sees (1,0) [same block]
            // (1,0) sees (0,0) [same column, same block]
            // All three are in block 0, forming a triangle.
            let chain_candidates: Candidates<Base3> =
                Candidates::try_from(vec![3, 7]).unwrap();

            let pos1: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            let pos2: Position<Base3> = (0, 1).try_into().unwrap();
            let pos3: Position<Base3> = (1, 0).try_into().unwrap();
            // Add a 4th to make component size ≥ 4, but isolated
            let pos4: Position<Base3> = (6u8, 6u8).try_into().unwrap();
            grid[pos1] = Cell::with_candidates(chain_candidates);
            grid[pos2] = Cell::with_candidates(chain_candidates);
            grid[pos3] = Cell::with_candidates(chain_candidates);
            grid[pos4] = Cell::with_candidates(chain_candidates);

            // Fill rest
            for pos in Position::<Base3>::all() {
                if grid.get(pos).candidates().is_none() && !grid.get(pos).has_value() {
                    let other_candidates: Candidates<Base3> =
                        Candidates::try_from(vec![1, 2, 4, 5, 6, 8, 9]).unwrap();
                    grid[pos] = Cell::with_candidates(other_candidates);
                }
            }

            let deductions = ChuteRemotePairs.execute(&grid).unwrap();
            // The 3-cell odd cycle component is not bipartite, so no deductions from it.
            // The 4th isolated cell forms a 1-cell component, also no deductions.
            assert!(
                deductions.is_empty(),
                "Should not find remote pairs with odd cycle components"
            );
        }
    }

    strategy_snapshot_tests!(ChuteRemotePairs);
}
