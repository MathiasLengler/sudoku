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

/// X-Cycles is an advanced chaining technique that uses alternating strong and weak links
/// for a single candidate digit to form cycles (loops).
///
/// Key concepts:
/// - **Strong Link**: Two cells in a unit where only those two can contain the candidate.
///   If one is false, the other must be true.
/// - **Weak Link**: Two cells in a unit where if one is true, the other must be false
///   (but both could be false).
///
/// The technique finds three types of cycles:
/// 1. **Nice Loop (Continuous)**: Alternates perfectly between strong and weak links.
///    Any cell seeing both ends of a weak link can have the candidate eliminated.
/// 2. **Discontinuous Loop with Weak Link**: Starts/ends at same cell with weak links.
///    The candidate in that cell can be eliminated.
/// 3. **Discontinuous Loop with Strong Link**: Starts/ends at same cell with strong links.
///    The candidate in that cell must be the value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct XCycles;

impl Strategy for XCycles {
    fn name(self) -> &'static str {
        "XCycles"
    }

    fn score(self) -> StrategyScore {
        350
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(Value::<Base>::all()
            .flat_map(|candidate| find_x_cycles_for_candidate(grid, candidate))
            .collect())
    }
}

/// Represents a link between two cells for a specific candidate.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum LinkType {
    /// Strong link: exactly two cells in the unit have this candidate.
    /// If one is false, the other must be true.
    Strong,
    /// Weak link: more than two cells in the unit have this candidate.
    /// If one is true, the other must be false.
    Weak,
}

impl LinkType {
    fn opposite(self) -> Self {
        match self {
            LinkType::Strong => LinkType::Weak,
            LinkType::Weak => LinkType::Strong,
        }
    }
}

/// A node in the X-Cycles graph (a cell position).
type Node<Base> = Position<Base>;

/// An edge in the X-Cycles graph.
#[derive(Debug, Clone)]
struct Edge<Base: SudokuBase> {
    target: Node<Base>,
    link_type: LinkType,
}

/// Graph of candidate positions with strong/weak links.
#[derive(Debug)]
struct CandidateGraph<Base: SudokuBase> {
    /// Maps each node to its adjacent edges.
    adjacency: BTreeMap<Node<Base>, Vec<Edge<Base>>>,
}

impl<Base: SudokuBase> CandidateGraph<Base> {
    fn new() -> Self {
        Self {
            adjacency: BTreeMap::new(),
        }
    }

    fn add_edge(&mut self, from: Node<Base>, to: Node<Base>, link_type: LinkType) {
        self.adjacency
            .entry(from)
            .or_default()
            .push(Edge { target: to, link_type });
    }

    fn nodes(&self) -> impl Iterator<Item = Node<Base>> + '_ {
        self.adjacency.keys().copied()
    }

    fn edges(&self, node: Node<Base>) -> impl Iterator<Item = &Edge<Base>> {
        self.adjacency.get(&node).into_iter().flat_map(|v| v.iter())
    }
}

/// Builds a graph of strong and weak links for a candidate.
fn build_candidate_graph<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
) -> CandidateGraph<Base> {
    let mut graph = CandidateGraph::new();

    // Find positions with this candidate
    let positions_with_candidate: Vec<Position<Base>> = grid
        .all_candidates_positions()
        .into_iter()
        .filter(|&pos| {
            grid.get(pos)
                .candidates()
                .is_some_and(|c| c.has(candidate))
        })
        .collect();

    // Build graph edges by checking rows, columns, and blocks
    add_links_for_rows(&mut graph, &positions_with_candidate);
    add_links_for_columns(&mut graph, &positions_with_candidate);
    add_links_for_blocks(&mut graph, &positions_with_candidate);

    graph
}

/// Add links between cells in the same row.
fn add_links_for_rows<Base: SudokuBase>(
    graph: &mut CandidateGraph<Base>,
    positions: &[Position<Base>],
) {
    // Group by row
    let mut rows: BTreeMap<Coordinate<Base>, Vec<Position<Base>>> = BTreeMap::new();
    for &pos in positions {
        rows.entry(pos.to_row()).or_default().push(pos);
    }

    for (_row, row_positions) in rows {
        add_links_for_unit(graph, &row_positions);
    }
}

/// Add links between cells in the same column.
fn add_links_for_columns<Base: SudokuBase>(
    graph: &mut CandidateGraph<Base>,
    positions: &[Position<Base>],
) {
    // Group by column
    let mut columns: BTreeMap<Coordinate<Base>, Vec<Position<Base>>> = BTreeMap::new();
    for &pos in positions {
        columns.entry(pos.to_column()).or_default().push(pos);
    }

    for (_col, col_positions) in columns {
        add_links_for_unit(graph, &col_positions);
    }
}

/// Add links between cells in the same block.
fn add_links_for_blocks<Base: SudokuBase>(
    graph: &mut CandidateGraph<Base>,
    positions: &[Position<Base>],
) {
    // Group by block
    let mut blocks: BTreeMap<Coordinate<Base>, Vec<Position<Base>>> = BTreeMap::new();
    for &pos in positions {
        blocks.entry(pos.to_block()).or_default().push(pos);
    }

    for (_block, block_positions) in blocks {
        add_links_for_unit(graph, &block_positions);
    }
}

/// Add links for all cells in a unit (row, column, or block).
fn add_links_for_unit<Base: SudokuBase>(
    graph: &mut CandidateGraph<Base>,
    positions: &[Position<Base>],
) {
    if positions.len() < 2 {
        return;
    }

    let link_type = if positions.len() == 2 {
        LinkType::Strong
    } else {
        LinkType::Weak
    };

    // Add bidirectional edges between all pairs
    for (i, &from) in positions.iter().enumerate() {
        for &to in positions.iter().skip(i + 1) {
            graph.add_edge(from, to, link_type);
            graph.add_edge(to, from, link_type);
        }
    }
}

/// Represents a path in the graph with alternating link types.
#[derive(Debug, Clone)]
struct AlternatingPath<Base: SudokuBase> {
    nodes: Vec<Node<Base>>,
    links: Vec<LinkType>,
}

impl<Base: SudokuBase> AlternatingPath<Base> {
    fn new(start: Node<Base>) -> Self {
        Self {
            nodes: vec![start],
            links: vec![],
        }
    }

    fn last_node(&self) -> Node<Base> {
        *self.nodes.last().unwrap()
    }

    fn last_link(&self) -> Option<LinkType> {
        self.links.last().copied()
    }

    fn contains(&self, node: Node<Base>) -> bool {
        self.nodes.contains(&node)
    }

    fn extend(&self, node: Node<Base>, link_type: LinkType) -> Self {
        let mut new_path = self.clone();
        new_path.nodes.push(node);
        new_path.links.push(link_type);
        new_path
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

/// Find X-Cycles for a specific candidate and return deductions.
fn find_x_cycles_for_candidate<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
) -> Vec<Deduction<Base>> {
    let graph = build_candidate_graph(grid, candidate);
    let mut deductions = Vec::new();

    // For each starting node, search for cycles
    for start in graph.nodes() {
        let mut visited_paths: BTreeSet<Vec<Position<Base>>> = BTreeSet::new();

        // Try starting with both strong and weak links
        for first_edge in graph.edges(start) {
            let initial_path = AlternatingPath::new(start).extend(first_edge.target, first_edge.link_type);
            find_cycles_dfs(
                grid,
                candidate,
                &graph,
                start,
                &initial_path,
                &mut deductions,
                &mut visited_paths,
            );
        }
    }

    deductions
}

/// Depth-first search to find cycles with alternating links.
fn find_cycles_dfs<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    graph: &CandidateGraph<Base>,
    start: Node<Base>,
    path: &AlternatingPath<Base>,
    deductions: &mut Vec<Deduction<Base>>,
    visited_paths: &mut BTreeSet<Vec<Position<Base>>>,
) {
    let current = path.last_node();
    let last_link = path.last_link();

    // Check for cycles back to start - need at least 4 nodes for a valid X-Cycle
    if path.len() >= 4 {
        for edge in graph.edges(current) {
            if edge.target == start {
                // Found a potential cycle
                let first_link = path.links.first().copied().unwrap();
                let closing_link = edge.link_type;

                // Normalize path for deduplication
                let mut normalized: Vec<_> = path.nodes.clone();
                normalized.sort();
                if visited_paths.contains(&normalized) {
                    continue;
                }
                visited_paths.insert(normalized);

                // Determine cycle type and apply rules
                if let Some(deduction) = process_cycle(grid, candidate, path, first_link, closing_link) {
                    deductions.push(deduction);
                }
            }
        }
    }

    // Continue DFS with alternating link type
    if let Some(required_link_type) = last_link.map(LinkType::opposite) {
        for edge in graph.edges(current) {
            // Must alternate link types
            if edge.link_type != required_link_type {
                continue;
            }
            // Don't revisit nodes (except start for cycle detection, which is handled above)
            if path.contains(edge.target) {
                continue;
            }
            // Limit path length to avoid exponential blowup
            if path.len() >= 12 {
                continue;
            }

            let new_path = path.extend(edge.target, edge.link_type);
            find_cycles_dfs(grid, candidate, graph, start, &new_path, deductions, visited_paths);
        }
    }
}

/// Process a found cycle and return a deduction if applicable.
fn process_cycle<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    path: &AlternatingPath<Base>,
    first_link: LinkType,
    closing_link: LinkType,
) -> Option<Deduction<Base>> {
    // Check if last_link alternates properly with closing_link
    let last_link = path.last_link()?;

    // For a valid cycle, we need to check if it's continuous (alternating) or discontinuous
    // A continuous loop: the closing_link alternates with last_link AND first_link
    // A discontinuous loop: the closing_link is the same as either last_link or first_link

    // Type 1: Nice Loop (Continuous) - alternates perfectly throughout
    // For this, closing_link must alternate with both its neighbors (last_link and first_link)
    if closing_link != last_link && closing_link != first_link {
        // This is a nice loop - find eliminations
        return find_nice_loop_eliminations(grid, candidate, path, closing_link);
    }

    // Type 2: Discontinuous Loop - the closing_link breaks the alternation at the start node
    // If first_link and closing_link are the same, we have a discontinuity at the start

    // If both are weak: The start node can be eliminated
    // (The chain proves that if the start is true, it leads to a contradiction)
    if first_link == LinkType::Weak && closing_link == LinkType::Weak && last_link == LinkType::Strong {
        return Some(create_elimination_deduction(candidate, path.nodes[0], &path.nodes));
    }

    // If both are strong: The start node must be true
    // (The chain proves the start node is forced)
    if first_link == LinkType::Strong && closing_link == LinkType::Strong && last_link == LinkType::Weak {
        return Some(create_value_deduction(candidate, path.nodes[0], &path.nodes));
    }

    None
}

/// Find eliminations from a nice loop (continuous cycle).
/// Any cell that can see two nodes connected by a weak link can have the candidate eliminated.
fn find_nice_loop_eliminations<Base: SudokuBase>(
    grid: &Grid<Base>,
    candidate: Value<Base>,
    path: &AlternatingPath<Base>,
    closing_link: LinkType,
) -> Option<Deduction<Base>> {
    let mut eliminations: Vec<Position<Base>> = Vec::new();

    // Find weak links in the cycle and check for eliminations
    // Include the closing link
    let all_links = path.links.iter().copied().chain(std::iter::once(closing_link));
    let all_edges: Vec<_> = path.nodes.iter().copied()
        .zip(path.nodes.iter().copied().cycle().skip(1).take(path.len()))
        .zip(all_links)
        .collect();

    for ((from, to), link_type) in all_edges {
        if link_type == LinkType::Weak {
            // Find cells that see both from and to
            for pos in grid.all_candidates_positions() {
                if pos == from || pos == to {
                    continue;
                }
                if path.nodes.contains(&pos) {
                    continue;
                }
                if !grid.get(pos).candidates().is_some_and(|c| c.has(candidate)) {
                    continue;
                }
                if sees_both(pos, from, to) {
                    eliminations.push(pos);
                }
            }
        }
    }

    if eliminations.is_empty() {
        return None;
    }

    // Remove duplicates
    eliminations.sort();
    eliminations.dedup();

    Some(
        Deduction::try_from_iters(
            eliminations.into_iter().map(|pos| (pos, Action::delete_candidate(candidate))),
            path.nodes.iter().map(|&pos| (pos, Reason::candidate(candidate))),
        )
        .unwrap(),
    )
}

/// Check if a position can see both target positions (shares a unit with both).
fn sees_both<Base: SudokuBase>(
    pos: Position<Base>,
    target1: Position<Base>,
    target2: Position<Base>,
) -> bool {
    sees(pos, target1) && sees(pos, target2)
}

/// Check if two positions share a unit (row, column, or block).
fn sees<Base: SudokuBase>(pos1: Position<Base>, pos2: Position<Base>) -> bool {
    pos1.to_row() == pos2.to_row()
        || pos1.to_column() == pos2.to_column()
        || pos1.to_block() == pos2.to_block()
}

/// Create a deduction to eliminate a candidate from a cell.
fn create_elimination_deduction<Base: SudokuBase>(
    candidate: Value<Base>,
    target: Position<Base>,
    chain: &[Position<Base>],
) -> Deduction<Base> {
    Deduction::try_from_iters(
        std::iter::once((target, Action::delete_candidate(candidate))),
        chain.iter().map(|&pos| (pos, Reason::candidate(candidate))),
    )
    .unwrap()
}

/// Create a deduction to set a value in a cell.
fn create_value_deduction<Base: SudokuBase>(
    candidate: Value<Base>,
    target: Position<Base>,
    chain: &[Position<Base>],
) -> Deduction<Base> {
    Deduction::try_from_iters(
        std::iter::once((target, Action::set_value(candidate))),
        chain
            .iter()
            .filter(|&&pos| pos != target)
            .map(|&pos| (pos, Reason::candidate(candidate))),
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::consts::*,
        cell::Cell,
        solver::strategic::strategies::test_util::strategy_snapshot_tests,
    };
    use indoc::indoc;

    mod synthetic {
        use super::*;

        /// Test that X-Cycles strategy executes without errors on a grid with candidates
        #[test]
        fn test_x_cycles_executes_without_error() {
            // A 4x4 grid with candidate patterns
            let mut grid: Grid<Base2> = indoc! {"
            1100
            0010
            0100
            0001"
            }
            .parse()
            .unwrap();
            
            // Convert to candidates
            for pos in grid.all_value_positions() {
                grid[pos] = Cell::with_candidates(grid[pos].to_candidates());
            }

            // Verify the strategy executes successfully
            let result = XCycles.execute(&grid);
            assert!(result.is_ok(), "XCycles should execute without error");
        }

        /// Test that X-Cycles finds no deductions when there are no cycles
        #[test]
        fn test_no_cycles() {
            let mut grid: Grid<Base2> = indoc! {"
            1000
            0000
            0000
            0001"
            }
            .parse()
            .unwrap();

            for pos in grid.all_value_positions() {
                grid[pos] = Cell::with_candidates(grid[pos].to_candidates());
            }

            let deductions = XCycles.execute(&grid).unwrap();

            // With only two isolated cells, no cycle is possible
            assert!(deductions.is_empty(), "No cycles should be found with isolated cells");
        }
    }

    mod link_detection {
        use super::*;

        #[test]
        fn test_build_candidate_graph() {
            let mut grid: Grid<Base2> = indoc! {"
            1100
            0010
            0100
            0001"
            }
            .parse()
            .unwrap();

            for pos in grid.all_value_positions() {
                grid[pos] = Cell::with_candidates(grid[pos].to_candidates());
            }

            let candidate = Value::<Base2>::try_from(1).unwrap();
            let graph = build_candidate_graph(&grid, candidate);

            // Check that graph has expected nodes
            let nodes: Vec<_> = graph.nodes().collect();
            assert!(!nodes.is_empty(), "Graph should have nodes");
        }

        #[test]
        fn test_sees_function() {
            type Base = Base3;
            
            // Same row
            let pos1: Position<Base> = (0, 0).try_into().unwrap();
            let pos2: Position<Base> = (0, 5).try_into().unwrap();
            assert!(sees(pos1, pos2), "Cells in same row should see each other");

            // Same column
            let pos3: Position<Base> = (3, 0).try_into().unwrap();
            assert!(sees(pos1, pos3), "Cells in same column should see each other");

            // Same block
            let pos4: Position<Base> = (1, 1).try_into().unwrap();
            assert!(sees(pos1, pos4), "Cells in same block should see each other");

            // Different row, column, and block
            let pos5: Position<Base> = (4, 4).try_into().unwrap();
            assert!(!sees(pos1, pos5), "Cells in different units should not see each other");
        }
    }

    strategy_snapshot_tests!(XCycles);
}
