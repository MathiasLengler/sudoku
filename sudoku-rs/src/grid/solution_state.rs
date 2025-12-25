use crate::{base::SudokuBase, grid::Grid, solver::introspective};
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "Base: SudokuBase")]
pub enum SolutionState<Base: SudokuBase> {
    NoSolution,
    MultipleSolutions,
    UniqueSolution(Grid<Base>),
}

impl<Base: SudokuBase> SolutionState<Base> {
    pub fn find_solution(grid: &Grid<Base>) -> Self {
        let mut solver = introspective::Solver::new(grid);

        let Some(first_solution) = solver.next() else {
            return Self::NoSolution;
        };

        if let Some(_second_solution) = solver.next() {
            Self::MultipleSolutions
        } else {
            Self::UniqueSolution(first_solution)
        }
    }

    pub fn into_unique_solution(self) -> Option<Grid<Base>> {
        if let SolutionState::UniqueSolution(grid) = self {
            Some(grid)
        } else {
            None
        }
    }

    pub fn as_unique_solution(&self) -> Option<&Grid<Base>> {
        if let SolutionState::UniqueSolution(grid) = self {
            Some(grid)
        } else {
            None
        }
    }

    pub fn is_unique(&self) -> bool {
        matches!(self, SolutionState::UniqueSolution(_))
    }
}
