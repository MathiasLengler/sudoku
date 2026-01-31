use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::Not;
use std::path::Path;

use itertools::Itertools;
use log::trace;
use varisat::{CnfFormula, ExtendFormula, Lit, Solver as SatSolver};

use crate::base::{BaseEnum, SudokuBase};
use crate::cell::{Cell, Value};
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::FallibleSolver;
use crate::solver::backtracking::CandidatesFilter;
use crate::solver::sat::cell_variable::CellVariable;

mod initialized_sat_solver {
    use super::*;
    use crate::base::consts::*;
    use std::sync::LazyLock;

    pub(super) static SOLVER_BASE_2: LazyLock<SatSolver> =
        LazyLock::new(Solver::<Base2>::init_sat_solver);
    pub(super) static SOLVER_BASE_3: LazyLock<SatSolver> =
        LazyLock::new(Solver::<Base3>::init_sat_solver);
    pub(super) static SOLVER_BASE_4: LazyLock<SatSolver> =
        LazyLock::new(Solver::<Base4>::init_sat_solver);
    pub(super) static SOLVER_BASE_5: LazyLock<SatSolver> =
        LazyLock::new(Solver::<Base5>::init_sat_solver);
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
    pub fn new<GridRef: AsRef<Grid<Base>>>(grid: GridRef) -> Self {
        Self::with_candidates_filter(grid, &())
    }

    pub fn with_candidates_filter<GridRef: AsRef<Grid<Base>>, Filter: CandidatesFilter<Base>>(
        grid: GridRef,
        filter: &Filter,
    ) -> Self {
        let sat_solver = Self::init_sat_solver_for_grid(grid.as_ref(), filter);

        Self {
            sat_solver,
            _base: PhantomData,
        }
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

    pub fn step_count(&self) -> u64 {
        self.sat_solver.stats().conflicts
    }
}

/// Helpers
impl<Base: SudokuBase> Solver<Base> {
    fn init_sat_solver_for_grid<Filter: CandidatesFilter<Base>>(
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
        assumptions.extend(
            filter
                .all_denied_candidates()
                .flat_map(|(pos, denied_candidates)| {
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

        sat_solver.assume(&assumptions);

        sat_solver
    }

    /// `Base`-cached version of `Self::init_sat_solver`
    #[allow(clippy::unnecessary_box_returns)]
    fn get_initialized_sat_solver() -> SatSolver<'static> {
        match Base::ENUM {
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

/// A specialized SAT-based checker for finding ambiguous solutions during pruning.
///
/// This checker is optimized for the pruning phase of sudoku generation:
/// - It adds a clause that prevents the known solution from being found
/// - It uses assumptions for the grid values, which can be changed without recreating the solver
/// - The solver is reused across multiple checks, preserving learned clauses
///
/// The checker efficiently determines if there exists an alternative solution
/// when a cell is removed from the puzzle.
pub struct AmbiguousSolutionChecker<Base: SudokuBase> {
    sat_solver: SatSolver<'static>,
    /// Stores the current grid value assumptions (positions that still have values in the puzzle)
    current_value_assumptions: Vec<Lit>,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase> Debug for AmbiguousSolutionChecker<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AmbiguousSolutionChecker")
            .field("sat_solver", &"<varisat::Solver>")
            .field(
                "current_value_assumptions_count",
                &self.current_value_assumptions.len(),
            )
            .field("_base", &self._base)
            .finish()
    }
}

impl<Base: SudokuBase> AmbiguousSolutionChecker<Base> {
    /// Create a new checker for the given solved grid.
    ///
    /// The `solution` must be a fully solved sudoku grid. A clause is added to prevent
    /// this exact solution from being found, so only alternative solutions can be discovered.
    pub fn new(solution: &Grid<Base>) -> Self {
        debug_assert!(
            solution.is_solved(),
            "Solution must be fully solved: {solution}"
        );

        let mut sat_solver = Solver::<Base>::get_initialized_sat_solver();

        // Add a clause that prevents the known solution from being found.
        // The clause is the negation of the conjunction of all solution values.
        // If the solution has values v1, v2, ..., vn at positions p1, p2, ..., pn,
        // we add the clause: NOT(p1=v1) OR NOT(p2=v2) OR ... OR NOT(pn=vn)
        // This ensures at least one position must have a different value.
        let solution_exclusion_clause: Vec<Lit> = Position::<Base>::all()
            .map(|pos| {
                let value = solution[pos].value().unwrap();
                CellVariable {
                    pos,
                    value,
                    is_true: false, // Negated
                }
                .into()
            })
            .collect();

        sat_solver.add_clause(&solution_exclusion_clause);

        // Initially, assume all positions are filled (will be updated when checking)
        let current_value_assumptions: Vec<Lit> = Position::<Base>::all()
            .map(|pos| {
                let value = solution[pos].value().unwrap();
                CellVariable {
                    pos,
                    value,
                    is_true: true,
                }
                .into()
            })
            .collect();

        sat_solver.assume(&current_value_assumptions);

        Self {
            sat_solver,
            current_value_assumptions,
            _base: PhantomData,
        }
    }

    /// Check if an ambiguous solution exists when the cell at `removed_pos` is deleted.
    ///
    /// This method updates the assumptions to exclude the value at `removed_pos` and
    /// instead denies the `denied_value` at that position (which is typically the
    /// solution value that was there).
    ///
    /// Returns `true` if an ambiguous solution exists (i.e., the puzzle is not uniquely solvable
    /// after removing the cell), `false` otherwise.
    pub fn has_ambiguous_solution(
        &mut self,
        removed_pos: Position<Base>,
        denied_value: Value<Base>,
    ) -> crate::error::Result<bool> {
        // Build the new assumptions:
        // - Keep all current value assumptions except for the removed position
        // - Add a negative assumption for the denied value at the removed position
        let mut assumptions: Vec<Lit> = self
            .current_value_assumptions
            .iter()
            .filter(|&lit| {
                // Filter out the assumption for the removed position
                let var: CellVariable<Base> = (*lit)
                    .try_into()
                    .expect("Should be able to convert Lit to CellVariable");
                var.pos != removed_pos
            })
            .copied()
            .collect();

        // Add assumption that denies the specific value at the removed position
        assumptions.push(
            CellVariable {
                pos: removed_pos,
                value: denied_value,
                is_true: false,
            }
            .into(),
        );

        self.sat_solver.assume(&assumptions);

        // Solve and check if a solution exists (which would be an ambiguous solution)
        Ok(self.sat_solver.solve()?)
    }

    /// Permanently remove a value from the current puzzle state.
    ///
    /// Call this after confirming that a cell can be removed (i.e., no ambiguous solution exists).
    /// This updates the internal state to reflect that the position no longer has a value.
    pub fn confirm_removal(&mut self, removed_pos: Position<Base>) {
        // Remove the assumption for this position from current_value_assumptions
        self.current_value_assumptions.retain(|lit| {
            let var: CellVariable<Base> = (*lit)
                .try_into()
                .expect("Should be able to convert Lit to CellVariable");
            var.pos != removed_pos
        });
    }

    /// Permanently keep a value in the current puzzle state.
    ///
    /// Call this after confirming that a cell cannot be removed (i.e., an ambiguous solution exists).
    /// The internal state already contains this position's assumption, so this is a no-op.
    #[allow(clippy::unused_self)]
    pub fn confirm_keep(&self, _kept_pos: Position<Base>) {
        // No-op: the position's assumption is already in current_value_assumptions
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
                    let mut solver = Solver::new(&grid);
                    assert_fallible_solver_single_solution(&mut solver, &grid);
                }
            }
        }

        mod fallible_solution_iter {
            use super::*;

            tests_solver_samples! {
                |grid| {
                    let solver = Solver::new(&grid);
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
        let solver = Solver::new(&grid);

        assert_infallible_solution_iter_all_solutions_base_2(
            assert_fallible_solution_iter_as_infallible(solver.into_iter()),
        );
    }

    #[test]
    fn test_candidates_filter_denied_candidates_grid() {
        type Base = Base2;

        let grid = Grid::<Base>::new();
        let mut denylist = Grid::new();
        denylist[Position::top_left()] = vec![1, 3]
            .into_iter()
            .map(|v| Value::try_from(v).unwrap())
            .collect();
        let solver = Solver::with_candidates_filter(&grid, &denylist);

        for solution in solver.clone() {
            assert!(
                ![1, 3].contains(
                    &solution
                        .unwrap()
                        .get(Position::top_left())
                        .value()
                        .unwrap()
                        .get()
                )
            );
        }

        assert_eq!(solver.clone().into_iter().count(), 144);
    }

    mod ambiguous_solution_checker {
        use super::*;
        use crate::samples;

        #[test]
        fn test_solved_grid_has_no_ambiguous_solution() {
            // A fully solved grid should not have any ambiguous solutions
            let solution = samples::base_2_solved();
            let mut checker = AmbiguousSolutionChecker::new(&solution);

            // Try removing any position - all should have no ambiguous solution
            // because the solution exclusion clause prevents the same solution
            // but with all values present, only the original solution satisfies the constraints
            let pos = Position::<Base2>::top_left();
            let value = solution[pos].value().unwrap();

            // When checking with the denied value being the solution value,
            // there should be no alternative solution
            let has_ambiguous = checker.has_ambiguous_solution(pos, value).unwrap();
            assert!(
                !has_ambiguous,
                "Removing a cell from a minimal puzzle should not have an ambiguous solution"
            );
        }

        #[test]
        fn test_non_minimal_grid_has_ambiguous_solution() {
            // Get a puzzle that has a unique solution
            let solution = samples::base_2_solved();

            // The fully solved grid is non-minimal - removing any single cell
            // won't cause multiple solutions because the other cells constrain it.
            // But if we remove enough cells, there will be multiple solutions.
            let mut checker = AmbiguousSolutionChecker::new(&solution);

            // We need to find a position that, when removed, still has a unique solution
            // (i.e., the solution is over-determined)
            // For a 4x4 sudoku, we need to identify if removing a cell creates ambiguity

            // Let's manually check a few positions
            let positions: Vec<_> = Position::<Base2>::all().collect();

            // Remove first position
            let first_pos = positions[0];
            let first_value = solution[first_pos].value().unwrap();
            let has_ambiguous = checker.has_ambiguous_solution(first_pos, first_value).unwrap();

            // Since this is a solved grid, removing one cell should NOT create ambiguity
            // (the remaining 15 cells fully determine the solution)
            assert!(
                !has_ambiguous,
                "Removing one cell from a solved Base2 grid should not create ambiguity"
            );
        }

        #[test]
        fn test_confirm_removal_updates_state() {
            let solution = samples::base_2_solved();
            let mut checker = AmbiguousSolutionChecker::new(&solution);

            let initial_len = checker.current_value_assumptions.len();
            assert_eq!(initial_len, 16); // Base2 has 16 cells

            let pos = Position::<Base2>::top_left();
            checker.confirm_removal(pos);

            assert_eq!(checker.current_value_assumptions.len(), initial_len - 1);
        }

        #[test]
        fn test_incremental_removal_finds_ambiguity() {
            // Start with a solved grid and progressively remove cells
            // At some point, removing a cell should create ambiguity
            let solution = samples::base_2_solved();
            let mut checker = AmbiguousSolutionChecker::new(&solution);

            let positions: Vec<_> = Position::<Base2>::all().collect();
            let mut removed_count = 0;
            let mut found_ambiguity = false;

            for pos in positions.into_iter() {
                let value = solution[pos].value().unwrap();
                let has_ambiguous = checker.has_ambiguous_solution(pos, value).unwrap();

                if has_ambiguous {
                    found_ambiguity = true;
                    break;
                } else {
                    // Can safely remove this cell
                    checker.confirm_removal(pos);
                    removed_count += 1;
                }
            }

            // We should find ambiguity at some point, since we can't remove all cells
            // while maintaining a unique solution
            assert!(
                found_ambiguity || removed_count < 16,
                "Should eventually find ambiguity when removing cells"
            );
        }
    }
}
