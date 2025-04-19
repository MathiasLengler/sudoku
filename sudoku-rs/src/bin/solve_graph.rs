use std::fs;

use petgraph::dot::{Config as DotConfig, Dot};
use petgraph::prelude::*;
use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::grid::Grid;
use sudoku::samples;
use sudoku::solver::strategic::strategies::StrategyEnum;
use sudoku::solver::strategic::{SolveStep, Solver};

fn main() -> Result<()> {
    // For each strategy and deduction:
    //  clone grid
    //  apply deduction
    //  add new grid to graph as node
    //  add edge from previous grid to new grid
    //  recurse, unitl all strategies are exhausted

    type Base = Base3;

    let mut graph = DiGraph::<Grid<Base>, StrategyEnum>::new();

    let grid = crate::samples::base_3().into_iter().nth(1).unwrap();

    let root_node_index = graph.add_node(grid.clone());

    let mut node_indexes_to_process = vec![root_node_index];

    while let Some(node_index) = node_indexes_to_process.pop() {
        let grid = &mut graph[node_index];
        let solver = Solver::new_with_strategies(
            grid,
            StrategyEnum::default_solver_strategies_no_brute_force(),
        );
        let all_deductions = solver.try_all_strategies().unwrap();

        let grid = graph[node_index].clone();

        for SolveStep {
            strategy,
            deductions,
        } in all_deductions
        {
            let mut new_grid = grid.clone();
            deductions.apply(&mut new_grid)?;

            if let Some(existing_node_index) = graph
                .raw_nodes()
                .iter()
                .position(|node| node.weight == new_grid)
                .map(NodeIndex::new)
            {
                // Grid already exists in graph
                graph.add_edge(node_index, existing_node_index, strategy);
            } else {
                let new_node_index = graph.add_node(new_grid);
                graph.add_edge(node_index, new_node_index, strategy);
                node_indexes_to_process.push(new_node_index);
            }
        }
    }

    dbg!(graph.node_count());
    dbg!(graph.edge_count());

    fs::write(
        "./sudoku-rs/out/solve_graph/graph.dot",
        format!("{}", Dot::with_config(&graph, &[DotConfig::NodeIndexLabel])),
    )?;

    Ok(())
}
