use std::marker::PhantomData;
use std::ops::Deref;

use anyhow::{anyhow, bail};
use itertools::Itertools;
use log::trace;
use splr::{Certificate, SatSolverIF, SolveIF};

use crate::base::{DynamicBase, SudokuBase};
use crate::cell::{Cell, Value};
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::sat::cell_variable::CellVariable;
use crate::solver::FallibleSolver;

mod initialized_sat_solver {
    use super::*;

    use crate::base::consts::*;
    use once_cell::sync::Lazy;

    pub(super) static SOLVER_BASE_2: Lazy<Box<splr::Solver>> =
        Lazy::new(|| Solver::<Base2>::init_sat_solver().unwrap());
    pub(super) static SOLVER_BASE_3: Lazy<Box<splr::Solver>> =
        Lazy::new(|| Solver::<Base3>::init_sat_solver().unwrap());
    pub(super) static SOLVER_BASE_4: Lazy<Box<splr::Solver>> =
        Lazy::new(|| Solver::<Base4>::init_sat_solver().unwrap());
    pub(super) static SOLVER_BASE_5: Lazy<Box<splr::Solver>> =
        Lazy::new(|| Solver::<Base5>::init_sat_solver().unwrap());
}

type Clause = Vec<i32>;

mod cell_variable;

#[derive(Debug)]
pub struct Solver<Base: SudokuBase> {
    sat_solver: Box<splr::Solver>,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase> Solver<Base> {
    pub fn new<GridRef: AsRef<Grid<Base>>>(grid: GridRef) -> Result<Self> {
        let sat_solver = Self::init_sat_solver_for_grid(grid.as_ref())?;

        Ok(Self {
            sat_solver,
            _base: PhantomData,
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    fn export_sat_solver_error(err: splr::SolverError) -> Error {
        anyhow!("SAT solver error: {}", err)
    }

    fn init_sat_solver_for_grid(grid: &Grid<Base>) -> Result<Box<splr::Solver>> {
        let mut sat_solver = Self::get_initialized_sat_solver();

        for pos in grid.all_value_positions() {
            let value = grid[pos].value().unwrap();

            sat_solver
                .add_assignment(
                    CellVariable {
                        pos,
                        value,
                        is_true: true,
                    }
                    .into(),
                )
                .map_err(Self::export_sat_solver_error)?;
        }

        Ok(sat_solver)
    }

    #[allow(clippy::unnecessary_box_returns)]
    fn get_initialized_sat_solver() -> Box<splr::Solver> {
        match Base::DYNAMIC_BASE {
            DynamicBase::Base2 => initialized_sat_solver::SOLVER_BASE_2.deref().clone(),
            DynamicBase::Base3 => initialized_sat_solver::SOLVER_BASE_3.deref().clone(),
            DynamicBase::Base4 => initialized_sat_solver::SOLVER_BASE_4.deref().clone(),
            DynamicBase::Base5 => initialized_sat_solver::SOLVER_BASE_5.deref().clone(),
        }
    }

    fn init_sat_solver() -> Result<Box<splr::Solver>> {
        let clauses = Self::general_clauses();
        let sat_solver = Box::new(
            match splr::Solver::try_from((splr::Config::default(), clauses.as_slice())) {
                Ok(s) => s,
                Err(Ok(Certificate::UNSAT)) => {
                    bail!("Grid is unsolvable")
                }
                Err(Ok(_)) => {
                    unreachable!("SAT solver should not return solution while initializing")
                }
                Err(Err(err)) => {
                    return Err(Self::export_sat_solver_error(err));
                }
            },
        );
        Ok(sat_solver)
    }

    // TODO: test helpers
    // TODO: implement other constraints from tdoku
    //  especially triad based-constraints
    /// All clauses which only depend on the base of the sudoku.
    ///
    /// Reference: [tdoku blog](https://t-dillon.github.io/tdoku/)
    fn general_clauses() -> Vec<Clause> {
        let mut clauses: Vec<Clause> = vec![];

        // Base clauses
        clauses.extend(Self::each_cell_contains_a_value_clauses());
        clauses.extend(Self::no_group_contains_the_same_value_twice_clauses());
        clauses.extend(Self::no_cell_contains_more_than_one_value_clauses());

        // Optimization clauses
        clauses.extend(Self::each_group_contains_each_value_clauses());

        clauses
    }

    /// Each cell contains a value
    ///
    /// Base3: 81 positive clauses, 9 literals each
    fn each_cell_contains_a_value_clauses() -> impl Iterator<Item = Clause> {
        Position::<Base>::all().map(|pos| {
            Value::<Base>::all()
                .map(|value| CellVariable {
                    pos,
                    value,
                    is_true: true,
                })
                .map(Into::into)
                .collect()
        })
    }

    /// No row|col|block contains the same value twice
    ///
    /// Base3: `3×81(9 choose 2)=8748` binary constraint clauses
    fn no_group_contains_the_same_value_twice_clauses() -> impl Iterator<Item = Clause> {
        Value::<Base>::all().flat_map(|value| {
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
        })
    }

    /// No cell contains more than one value
    ///
    /// Base3: `81(9 choose 2)=2916` binary constraint clauses
    fn no_cell_contains_more_than_one_value_clauses() -> impl Iterator<Item = Clause> {
        Position::<Base>::all().flat_map(|pos| {
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
        })
    }

    /// Each group contains each value
    ///
    /// Base3: 9 clauses, 9 literals each
    ///
    /// [tdoku reference](https://t-dillon.github.io/tdoku/#:~:text=new%20positive%20clauses%20that%20are%20group%2Daligned%20instead%20of%20cell%2Daligned)
    fn each_group_contains_each_value_clauses() -> impl Iterator<Item = Clause> {
        Value::<Base>::all().flat_map(|value| {
            Position::<Base>::all_groups().map(move |group| {
                group
                    .map(|pos| {
                        CellVariable {
                            pos,
                            value,
                            is_true: false,
                        }
                        .into()
                    })
                    .collect()
            })
        })
    }
}

impl<Base: SudokuBase> FallibleSolver<Base> for Solver<Base> {
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
