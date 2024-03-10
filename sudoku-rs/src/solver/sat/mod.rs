use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::Not;
use std::path::Path;

use itertools::Itertools;
use log::trace;
use varisat::{CnfFormula, ExtendFormula, Lit, Solver as SatSolver};

use crate::base::{BaseEnum, SudokuBase};
use crate::cell::{Candidates, Cell, Value};
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking::AvailabilityFilter;
use crate::solver::sat::cell_variable::CellVariable;
use crate::solver::FallibleSolver;

mod initialized_sat_solver {
    use once_cell::sync::Lazy;

    use crate::base::consts::*;

    use super::*;

    pub(super) static SOLVER_BASE_2: Lazy<SatSolver> = Lazy::new(Solver::<Base2>::init_sat_solver);
    pub(super) static SOLVER_BASE_3: Lazy<SatSolver> = Lazy::new(Solver::<Base3>::init_sat_solver);
    pub(super) static SOLVER_BASE_4: Lazy<SatSolver> = Lazy::new(Solver::<Base4>::init_sat_solver);
    pub(super) static SOLVER_BASE_5: Lazy<SatSolver> = Lazy::new(Solver::<Base5>::init_sat_solver);
}

type Clause = Vec<Lit>;

mod cell_variable;

#[derive(Clone)]
pub struct Solver<Base: SudokuBase> {
    sat_solver: SatSolver<'static>,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase> Debug for Solver<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Solver")
            .field("sat_solver", &"<missing>")
            .field("_base", &self._base)
            .finish()
    }
}

/// Public API
impl<Base: SudokuBase> Solver<Base> {
    pub fn new<GridRef: AsRef<Grid<Base>>>(grid: GridRef) -> Result<Self> {
        Self::new_with_availability_filter(grid, &())
    }

    pub fn new_with_availability_filter<
        GridRef: AsRef<Grid<Base>>,
        Filter: AvailabilityFilter<Base>,
    >(
        grid: GridRef,
        filter: &Filter,
    ) -> Result<Self> {
        let sat_solver = Self::init_sat_solver_for_grid(grid.as_ref(), filter);

        Ok(Self {
            sat_solver,
            _base: PhantomData,
        })
    }

    // Helpers for sat comparison
    pub fn dump_cnf(&self, _path: &Path) {
        todo!();
        // self.sat_solver.dump_cnf(path);
    }

    pub fn grid_assignments(grid: &Grid<Base>) -> Vec<i32> {
        grid.all_value_positions()
            .into_iter()
            .map(|pos| {
                let value = grid[pos].value().unwrap();

                CellVariable {
                    pos,
                    value,
                    is_true: true,
                }
                .into()
            })
            .collect()
    }
}

/// Helpers
impl<Base: SudokuBase> Solver<Base> {
    fn init_sat_solver_for_grid<Filter: AvailabilityFilter<Base>>(
        grid: &Grid<Base>,
        filter: &Filter,
    ) -> SatSolver<'static> {
        let mut sat_solver = Self::get_initialized_sat_solver();

        // Add grid assumptions
        let mut assumptions: Vec<Lit> = grid
            .all_value_positions()
            .into_iter()
            .map(|pos| {
                let value = grid[pos].value().unwrap();

                CellVariable {
                    pos,
                    value,
                    is_true: true,
                }
                .into()
            })
            .collect();

        // Add filter assumptions
        if !Filter::IS_NOOP {
            let all_candidates = Candidates::<Base>::all();

            assumptions.extend(
                Position::<Base>::all()
                    .filter_map(|pos| {
                        let remaining_candidates = filter.filter(all_candidates, pos.into());
                        if remaining_candidates == all_candidates {
                            None
                        } else {
                            Some((pos, remaining_candidates))
                        }
                    })
                    .flat_map(|(pos, remaining_candidates)| {
                        let denied_candidates = all_candidates.without(remaining_candidates);
                        denied_candidates.into_iter().map(move |denied_value| {
                            // Remove denied value via a negative assignment
                            Lit::from(CellVariable {
                                pos,
                                value: denied_value,
                                is_true: false,
                            })
                        })
                    }),
            );
        }

        sat_solver.assume(&assumptions);

        sat_solver
    }

    /// `Base`-cached version of `Self::init_sat_solver`
    #[allow(clippy::unnecessary_box_returns)]
    fn get_initialized_sat_solver() -> SatSolver<'static> {
        match Base::DYNAMIC_BASE {
            BaseEnum::Base2 => initialized_sat_solver::SOLVER_BASE_2.deref().clone(),
            BaseEnum::Base3 => initialized_sat_solver::SOLVER_BASE_3.deref().clone(),
            BaseEnum::Base4 => initialized_sat_solver::SOLVER_BASE_4.deref().clone(),
            BaseEnum::Base5 => initialized_sat_solver::SOLVER_BASE_5.deref().clone(),
        }
    }

    fn init_sat_solver() -> SatSolver<'static> {
        let clauses = Self::general_clauses();

        let mut formula = CnfFormula::new();

        for clause in clauses {
            formula.add_clause(&clause);
        }

        let mut sat_solver = SatSolver::new();

        sat_solver.add_formula(&formula);

        sat_solver
    }

    fn solve_with_assignment(&mut self) -> Result<Option<Vec<Lit>>> {
        Ok(self.sat_solver.solve()?.then(|| {
            self.sat_solver
                .model()
                .expect("SatSolver should return model on successful solve")
        }))
    }

    pub fn assigment_to_solution(assignment: &[Lit]) -> Result<Grid<Base>> {
        let variables = assignment
            .iter()
            .copied()
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

        Ok(solution)
    }
}

/// Clauses
impl<Base: SudokuBase> Solver<Base> {
    // TODO: test clauses
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
        Ok(if let Some(assignment) = self.solve_with_assignment()? {
            let solution = Self::assigment_to_solution(&assignment)?;
            Some(solution)
        } else {
            None
        })
    }
}

impl<Base: SudokuBase> IntoIterator for Solver<Base> {
    type Item = Result<Grid<Base>>;

    type IntoIter = SolverIter<Base>;

    fn into_iter(self) -> Self::IntoIter {
        SolverIter {
            solver: self,
            last_assignment: None,
        }
    }
}

#[derive(Debug)]
pub struct SolverIter<Base: SudokuBase> {
    solver: Solver<Base>,
    last_assignment: Option<Vec<Lit>>,
}

impl<Base: SudokuBase> Iterator for SolverIter<Base> {
    type Item = Result<Grid<Base>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(last_assignment) = self.last_assignment.take() {
            self.solver.sat_solver.add_clause(
                &last_assignment
                    .into_iter()
                    .map(Not::not)
                    .collect::<Vec<_>>(),
            );
        }

        Some(
            self.solver
                .solve_with_assignment()
                .transpose()?
                .and_then(|assignment| {
                    Solver::<Base>::assigment_to_solution(&assignment).inspect(|_| {
                        self.last_assignment = Some(assignment);
                    })
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;
    use crate::solver::test_util::{
        assert_fallible_solution_iter_as_infallible, assert_fallible_solver_single_solution,
        assert_infallible_solution_iter_all_solutions_base_2,
        assert_infallible_solution_iter_single_solution, tests_solver_samples,
    };
    use crate::test_util::init_test_logger;

    use super::*;

    mod samples {
        use super::*;

        mod fallible_solver {
            use super::*;

            tests_solver_samples! {
                init_test_logger(),
                |grid| {
                    let solver = Solver::new(&grid).unwrap();
                    assert_fallible_solver_single_solution(solver, &grid);
                }
            }
        }

        mod fallible_solution_iter {
            use super::*;

            tests_solver_samples! {
                |grid| {
                    let solver = Solver::new(&grid).unwrap();
                    assert_infallible_solution_iter_single_solution(
                        assert_fallible_solution_iter_as_infallible(solver.into_iter()), &grid
                    );
                }
            }
        }
    }

    #[test]
    fn test_iter_all_solutions() {
        let grid = Grid::<Base2>::new();
        let solver = Solver::new(&grid).unwrap();

        assert_infallible_solution_iter_all_solutions_base_2(
            assert_fallible_solution_iter_as_infallible(solver.into_iter()),
        );
    }

    #[test]
    fn test_availability_filter_denied_candidates_grid() {
        type Base = Base2;

        let grid = Grid::<Base>::new();
        let mut denylist = Grid::new();
        denylist[Position::top_left()] = vec![1, 3]
            .into_iter()
            .map(|v| v.try_into().unwrap())
            .collect();
        let solver = Solver::new_with_availability_filter(&grid, &denylist).unwrap();

        for solution in solver.clone() {
            assert!(![1, 3].contains(
                &solution
                    .unwrap()
                    .get(Position::top_left())
                    .value()
                    .unwrap()
                    .get()
            ));
        }

        assert_eq!(solver.clone().into_iter().count(), 144);
    }
}
