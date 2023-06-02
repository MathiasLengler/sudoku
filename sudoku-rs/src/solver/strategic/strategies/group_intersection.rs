use num::{One, PrimInt, Zero};

use crate::base::SudokuBase;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::{Coordinate, Position};
use crate::solver::backtracking_bitset::group_availability::CandidatesGroup;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::Strategy;

/// For a single candidate, where in each group is this candidate set?
#[derive(Debug, Clone, Default)]
struct GroupCandidateIndexes<Base: SudokuBase> {
    rows: CandidatesGroup<Base>,    // intersects with row_major_blocks
    columns: CandidatesGroup<Base>, // intersects with column_major_blocks
    row_major_blocks: CandidatesGroup<Base>, // intersects with rows
    column_major_blocks: CandidatesGroup<Base>, // intersects with columns
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
        let base = Base::BASE;

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
            .filter_map(|(coordinate, group_candidate_indexes)| {
                dbg!(coordinate);

                let candidate: Value<Base> = coordinate.into();

                // for (rows_i, row) in
                //     Coordinate::<Base>::all().zip(group_candidate_indexes.rows.iter())
                // {
                //     if let Some(segment_index) = row.base_segmentation() {
                //         group_candidate_indexes.row_major_blocks.get(
                //             Coordinate::new(segment_index + ((rows_i.get() / base) * base))
                //                 .unwrap(),
                //         );
                //     }
                // }

                for (block_i, block) in
                    Coordinate::<Base>::all().zip(group_candidate_indexes.row_major_blocks.iter())
                {
                    if let Some(segment_index) = block.base_segmentation() {
                        let row_i = (block_i.to_block_row(), segment_index).into();
                        group_candidate_indexes.rows.get(row_i);
                    }
                }

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
