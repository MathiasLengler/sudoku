use std::mem::replace;

use gcollections::ops::*;
use indexmap::IndexMap;
use interval::interval::ToInterval;
use pcp::concept::*;
use pcp::kernel::*;
use pcp::logic::Disjunction;
use pcp::propagation::events::FDEvent;
use pcp::propagation::PropagatorConcept;
use pcp::propagators::*;
use pcp::search::search_tree_visitor::Status::*;
use pcp::search::SearchTreeVisitor;
use pcp::search::*;
use pcp::term::*;
use pcp::variable::ops::*;

use pcp_utils::{boxed_one_solution_engine_interval, FDSpace, VStore};

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

pub(super) mod pcp_utils;

// TODO: compare perf with python Z3 implementation

pub struct Solver<'s, Cell: SudokuCell> {
    sudoku: &'s mut Sudoku<Cell>,

    variable_positions: Vec<Position>,

    // PCP state
    search: Box<dyn SearchTreeVisitor<FDSpace>>,
    space: FDSpace,
}

impl<'s, Cell: SudokuCell> Solver<'s, Cell> {
    pub fn new(sudoku: &'s mut Sudoku<Cell>) -> Solver<'s, Cell> {
        sudoku.fix_all_values();
        sudoku.set_all_direct_candidates();

        let (space, variable_positions) = Self::constrain(sudoku);

        let mut search = boxed_one_solution_engine_interval();

        search.start(&space);

        Self {
            sudoku,
            variable_positions,
            search,
            space,
        }
    }

    fn constrain(sudoku: &Sudoku<Cell>) -> (FDSpace, Vec<Position>) {
        // TODO: constrain all cells, not only the candidates
        //  simpler constraints could be faster

        let max_value = sudoku.grid().max_value() as i32;

        let mut space = FDSpace::empty();

        let pos_to_variable_and_candidates: IndexMap<_, _> = sudoku
            .grid()
            .all_positions()
            .filter_map(|pos| {
                sudoku.get(pos).candidates().map(|candidates| {
                    // Define variable constrained to range
                    let variable =
                        Box::new(space.vstore.alloc((1, max_value).to_interval())) as Var<VStore>;

                    (pos, (variable, candidates))
                })
            })
            .collect();

        // Variables must be one of the given candidates
        for (variable, candidates) in pos_to_variable_and_candidates.values() {
            for candidate in 1..=max_value {
                if !candidates.contains(&(candidate as usize)) {
                    space.cstore.alloc(Box::new(XNeqY::new(
                        variable.bclone(),
                        Box::new(Constant::new(candidate)),
                    )));
                }
            }
        }

        // Group constraints:
        constrain_groups(
            sudoku.grid().all_row_positions(),
            &mut space,
            &pos_to_variable_and_candidates,
        );
        constrain_groups(
            sudoku.grid().all_column_positions(),
            &mut space,
            &pos_to_variable_and_candidates,
        );
        constrain_groups(
            sudoku.grid().all_block_positions(),
            &mut space,
            &pos_to_variable_and_candidates,
        );

        let variable_positions = pos_to_variable_and_candidates
            .into_iter()
            .map(|(pos, _)| pos)
            .collect();

        (space, variable_positions)
    }

    pub fn try_solve(&mut self) -> bool {
        let space = replace(&mut self.space, FDSpace::empty());
        let (frozen_space, status) = self.search.enter(space);
        self.space = frozen_space.unfreeze();

        match status {
            Satisfiable => {
//                eprintln!("space.vstore = {:?}", self.space.vstore.iter().map(|dom| (dom.lower(), dom.upper())).collect::<Vec<_>>());

                // apply solution to sudoku
                for (dom, pos) in self.space.vstore.iter().zip(&self.variable_positions) {
                    let lower = dom.lower();
                    let upper = dom.upper();

                    assert_eq!(lower, upper);

                    self.sudoku.set_value(*pos, lower as usize);
                }

                true
            }
            Unsatisfiable => {
                false
            }
            EndOfSearch => {
                false
            }
            Unknown(_) => unreachable!(
                "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
        }
    }
}

fn constrain_groups(
    groups: impl Iterator<Item = impl Iterator<Item = Position>>,
    space: &mut FDSpace,
    pos_to_variable_and_candidates: &IndexMap<Position, (Var<VStore>, Vec<usize>)>,
) {
    for group in groups {
        let group_variables: Vec<_> = group
            .filter_map(|pos| {
                pos_to_variable_and_candidates
                    .get(&pos)
                    .map(|(variable, _)| variable.bclone())
            })
            .collect();

        if !group_variables.is_empty() {
            space.cstore.alloc(Box::new(Distinct::new(group_variables)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_2() {
        let mut sudokus = crate::samples::base_2();

        for (sudoku_index, mut sudoku) in sudokus.drain(1..2).enumerate() {
            println!("#{}:\n{}", sudoku_index, sudoku);

            let mut solver = Solver::new(&mut sudoku);

            assert!(solver.try_solve());

            assert!(sudoku.is_solved());
        }
    }

    #[test]
    fn test_base_3() {
        let sudokus = crate::samples::base_3();

        for (sudoku_index, mut sudoku) in sudokus.into_iter().enumerate() {
            println!("#{}:\n{}", sudoku_index, sudoku);

            let mut solver = Solver::new(&mut sudoku);

            assert!(solver.try_solve());

            assert!(sudoku.is_solved());
        }
    }

    #[test]
    fn test_minimal() {
        let mut sudoku = crate::samples::minimal(2);

        let mut solver = Solver::new(&mut sudoku);

        assert!(solver.try_solve());

        assert!(sudoku.is_solved());
    }
}
