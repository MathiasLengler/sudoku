use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::{Coordinate, Position};
use crate::solver::backtracking_bitset::group_availability::CandidatesGroup;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::Strategy;

/// For a single candidate, where in each group is this candidate set?
#[derive(Debug, Clone, Default)]
struct GroupCandidateIndexes<Base: SudokuBase> {
    rows: CandidatesGroup<Base>,
    columns: CandidatesGroup<Base>,
    row_major_blocks: CandidatesGroup<Base>,
    column_major_blocks: CandidatesGroup<Base>,
}

// TODO: split into separate strategies?
//  "Pointing Pairs, Pointing Triples"
//  "Box/Line Reduction"
//  decide after implementation, how much the algorithm differs.

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GroupIntersection;

impl Strategy for GroupIntersection {
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        // TODO: implement https://www.sudokuwiki.org/Intersection_Removal

        // TODO: use data structure Vec<GroupAvailability>

        let mut candidate_to_group_candidate_indexes =
            vec![GroupCandidateIndexes::<Base>::default(); usize::from(Base::SIDE_LENGTH)];

        for pos in Position::<Base>::all() {
            if let Some(candidates) = grid[pos].candidates() {
                for candidate in candidates {
                    let group_candidate_indexes = &mut candidate_to_group_candidate_indexes
                        [usize::from(candidate.into_u8() - 1)];

                    let row_index = pos.to_column().into();
                    group_candidate_indexes
                        .rows
                        .get_mut(pos.to_row())
                        .insert(row_index);
                    let column_index = pos.to_row().into();
                    group_candidate_indexes
                        .columns
                        .get_mut(pos.to_column())
                        .insert(column_index);

                    let (block, row_major_block_index, column_major_block_index) =
                        pos.to_block_and_indexes();

                    group_candidate_indexes
                        .row_major_blocks
                        .get_mut(block)
                        .insert(row_major_block_index.into());
                    group_candidate_indexes
                        .column_major_blocks
                        .get_mut(block)
                        .insert(column_major_block_index.into());
                }
            }
        }

        Coordinate::<Base>::all()
            .zip(candidate_to_group_candidate_indexes)
            .filter_map(|(group_candidate_indexes, coordinate)| {
                dbg!((coordinate, group_candidate_indexes));

                // TODO: find Base length strips in candidates bitset
                //  where all existing bits are contained in this section.
                //  Pass (middle group slice):
                //  - 000111000
                //  - 000101000
                //  - 000110000
                //  - 000011000
                //  Fail:
                //  - 010111000
                //  - 010101000
                //  - 000101010
                //  Fail: Second condition: at least two bits must be set.
                //  - 000100000

                None
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    #[test]
    fn test_execute() {
        // Intersection of value 1 in row 3/box 3, reduction in row 3/box 2
        let grid = Grid::<Base2>::try_from(
            "╔═══════════╦═══════════╗
║ 1   │     ║     │ 1   ║
║     │  2  ║  3  │     ║
║   4 │     ║     │   4 ║
║─────┼─────║─────┼─────║
║     │ 1   ║ 1 2 │ 1   ║
║  3  │     ║     │     ║
║     │   4 ║     │   4 ║
╠═══════════╬═══════════╣
║ 1   │ 1   ║     │     ║
║     │     ║  4  │  2  ║
║     │ 3   ║     │     ║
║─────┼─────║─────┼─────║
║ 1 2 │ 1   ║ 1   │ 1   ║
║     │     ║     │     ║
║   4 │ 3 4 ║     │ 3   ║
╚═══════════╩═══════════╝",
        )
        .unwrap();

        println!("{grid}");

        let deductions = GroupIntersection.execute(&grid).unwrap();

        println!("Deductions: {deductions}");

        // TODO: assert deductions
    }
}
