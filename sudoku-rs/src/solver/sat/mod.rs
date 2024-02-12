use std::marker::PhantomData;

use anyhow::bail;
use itertools::Itertools;
use log::trace;
use num::Zero;
use splr::{Certificate, SolveIF};

use crate::base::SudokuBase;
use crate::cell::{Cell, Value};
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::sat::cell_variable::CellVariable;
use crate::solver::FallibleSolver;

mod cell_variable;

#[derive(Debug)]
pub struct Solver<Base: SudokuBase, GridRef: AsRef<Grid<Base>>> {
    /// Grid to be solved
    grid: GridRef,
    sat_solver: Box<splr::Solver>,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>> Solver<Base, GridRef> {
    pub fn new(grid: GridRef) -> Result<Self> {
        let formula = Self::formula(grid.as_ref());
        dbg!(formula.len());
        let sat_solver = Box::new(
            match splr::Solver::try_from((splr::Config::default(), formula.as_slice())) {
                Ok(s) => s,
                Err(Ok(Certificate::UNSAT)) => {
                    bail!("Grid is unsolvable")
                }
                Err(Ok(_)) => {
                    unreachable!("SAT solver should not return solution while initializing")
                }
                Err(Err(err)) => {
                    bail!("SAT solver error while initializing: {}", err)
                }
            },
        );

        Ok(Self {
            sat_solver,
            grid,
            _base: PhantomData,
        })
    }

    fn grid(&self) -> &Grid<Base> {
        self.grid.as_ref()
    }

    // TODO: extract constraints into helpers
    // TODO: test helpers
    // TODO: evaluate caching/compile time construction/re-use of general constraints
    // TODO: implement other constraints from tdoku and benchmark
    // Reference: https://t-dillon.github.io/tdoku/
    fn formula(grid: &Grid<Base>) -> Vec<Vec<i32>> {
        let mut clauses: Vec<Vec<i32>> = vec![];

        // General constraints

        // Each cell contains a value (Base3: 81 positive clauses, 9 literals each)
        clauses.extend(Position::<Base>::all().map(|pos| {
            Value::<Base>::all()
                .map(|value| CellVariable {
                    pos,
                    value,
                    is_true: true,
                })
                .map(Into::into)
                .collect()
        }));

        // No row|col|block contains the same value twice (Base3: 3×81(9 choose 2)=8748 binary constraint clauses)
        clauses.extend(Value::<Base>::all().flat_map(|value| {
            Position::<Base>::all_groups().flat_map(move |group| {
                group.tuple_combinations().map(move |(pos1, pos2)| {
                    vec![
                        CellVariable {
                            pos: pos1,
                            value,
                            is_true: false,
                        }
                        .into(),
                        CellVariable {
                            pos: pos2,
                            value,
                            is_true: false,
                        }
                        .into(),
                    ]
                })
            })
        }));

        // No cell contains more than one value (Base3: 81(9 choose 2)=2916 binary constraint clauses)
        clauses.extend(Position::<Base>::all().flat_map(|pos| {
            Value::<Base>::all()
                .tuple_combinations()
                .map(move |(value1, value2)| {
                    vec![
                        CellVariable {
                            pos,
                            value: value1,
                            is_true: false,
                        }
                        .into(),
                        CellVariable {
                            pos,
                            value: value2,
                            is_true: false,
                        }
                        .into(),
                    ]
                })
        }));

        // Grid specific constraints
        clauses.extend(grid.all_value_positions().into_iter().map(|pos| {
            let value = grid[pos].value().unwrap();

            vec![CellVariable {
                pos,
                value,
                is_true: true,
            }
            .into()]
        }));

        clauses
    }
}

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>> FallibleSolver<Base> for Solver<Base, GridRef> {
    type Error = Error;

    fn try_solve(&mut self) -> Result<Option<Grid<Base>>> {
        match self.sat_solver.solve() {
            Ok(Certificate::SAT(assignment)) => {
                let variables = assignment
                    .into_iter()
                    .map(CellVariable::<Base>::try_from)
                    .collect::<Result<Vec<_>>>()?;

                trace!(
                    "Assigned cell variables:\n{}",
                    variables.iter().map(ToString::to_string).join("\n")
                );

                let true_cell_variables = variables.into_iter().filter(|var| var.is_true);

                // check if all positions have a true clause and are in ascending (row-major) order
                debug_assert!({
                    let positions = true_cell_variables.clone().map(|var| var.pos);
                    itertools::assert_equal(positions, Position::<Base>::all());
                    true
                });

                let solution = Grid::<Base>::with(
                    true_cell_variables
                        .map(|var| Cell::with_value(var.value, false))
                        .collect(),
                )?;

                debug_assert!(
                    solution.is_solved(),
                    "Solution should be solved:\n{solution}"
                );

                Ok(Some(solution))
            }
            Ok(Certificate::UNSAT) => Ok(None),
            Err(err) => {
                bail!("SAT solver error while solving: {}", err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::solver::test_util::{assert_fallible_solver_single_solution, tests_solver_samples};
    use crate::test_util::init_test_logger;

    use super::*;

    tests_solver_samples! {
        init_test_logger(),
        |grid| {
            let solver = Solver::new(&grid).unwrap();
            assert_fallible_solver_single_solution(solver, &grid);
        }
    }
}
