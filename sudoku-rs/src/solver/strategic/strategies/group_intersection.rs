use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::Strategy;

// TODO: split into separate strategies?
//  "Pointing Pairs, Pointing Triples"
//  "Box/Line Reduction"
//  decide after implementation, if how much the algorithm differs.

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GroupIntersection;

impl Strategy for GroupIntersection {
    fn execute<Base: SudokuBase>(self, _grid: &Grid<Base>) -> Result<Deductions<Base>> {
        // TODO: implement https://www.sudokuwiki.org/Intersection_Removal

        // TODO: use data structure Vec<GroupAvailability>

        todo!("GroupIntersection")
    }
}
