use itertools::izip;
use num::{One, PrimInt, Zero};

use crate::base::SudokuBase;
use crate::cell::{Candidates, Value};
use crate::error::Result;
use crate::grid::Grid;
use crate::position::{Coordinate, Position};
use crate::solver::backtracking_bitset::group_availability::CandidatesGroup;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};
use crate::solver::strategic::strategies::group_intersection::block_segment::{
    BlockSegment, CellOrder,
};
use crate::solver::strategic::strategies::Strategy;

mod block_segment;

/// For a single candidate, where in each group is this candidate set?
#[derive(Debug, Clone, Default)]
struct GroupCandidateIndexes<Base: SudokuBase> {
    // intersects with row_major_blocks
    rows: CandidatesGroup<Base>,
    // intersects with column_major_blocks
    columns: CandidatesGroup<Base>,
    // intersects with rows
    row_major_blocks: CandidatesGroup<Base>,
    // intersects with columns
    column_major_blocks: CandidatesGroup<Base>,
}

impl<Base: SudokuBase> GroupCandidateIndexes<Base> {
    fn with_grid(grid: &Grid<Base>) -> Vec<Self> {
        let mut candidate_to_group_candidate_indexes =
            vec![GroupCandidateIndexes::<Base>::default(); usize::from(Base::SIDE_LENGTH)];

        for pos in Position::<Base>::all() {
            if let Some(candidates) = grid[pos].candidates() {
                for candidate in candidates {
                    let group_candidate_indexes =
                        &mut candidate_to_group_candidate_indexes[usize::from(candidate.get() - 1)];

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
        candidate_to_group_candidate_indexes
    }

    fn get(&self, block_segment: BlockSegment<Base>) {
        // TODO: split into separate methods
        let (axis_candidate_positions, block_candidate_positions) = match block_segment.orientation
        {
            CellOrder::RowMajor => {
                let row_candidate_positions = self.rows.get(block_segment.axis());
                let block_candidate_positions = self.row_major_blocks.get(block_segment.block);

                (row_candidate_positions, block_candidate_positions)
            }
            CellOrder::ColumnMajor => {
                let column_candidate_positions = self.columns.get(block_segment.axis());
                let block_candidate_positions = self.column_major_blocks.get(block_segment.block);
                (column_candidate_positions, block_candidate_positions)
            }
        };

        match (
            axis_candidate_positions.block_segmentation(),
            block_candidate_positions.block_segmentation(),
        ) {
            (Some(axis_segment_index), None)
                if axis_segment_index == block_segment.axis_segment_index() =>
            {
                println!(
                    "{block_segment}: {:?} axis_segment_index {axis_segment_index}",
                    block_segment.orientation
                );
            }
            (None, Some(block_segment_index))
                if block_segment_index == block_segment.block_segment_index() =>
            {
                println!("{block_segment}: block_segment_index {block_segment_index}");
            }
            _ => {}
        }
    }
}

// TODO: split into separate strategies?
//  "Pointing Pairs, Pointing Triples" (segmented block)
//  "Box/Line Reduction" (segmented axis)
//  decide after implementation, how much the algorithm differs.

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GroupIntersection;

impl Strategy for GroupIntersection {
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        // TODO: implement https://www.sudokuwiki.org/Intersection_Removal

        let candidate_to_group_candidate_indexes = GroupCandidateIndexes::with_grid(grid);

        // TODO: replace return with this iterator
        izip!(Value::<Base>::all(), &candidate_to_group_candidate_indexes).for_each(
            |(candidate, group_candidate_indexes)| {
                dbg!(candidate);
                BlockSegment::<Base>::all(CellOrder::RowMajor).for_each(|block_segment| {
                    group_candidate_indexes.get(block_segment);
                });
                BlockSegment::<Base>::all(CellOrder::ColumnMajor).for_each(|block_segment| {
                    group_candidate_indexes.get(block_segment);
                });
            },
        );

        Ok(
            izip!(Value::<Base>::all(), candidate_to_group_candidate_indexes,)
                .flat_map(move |(candidate, group_candidate_indexes)| {
                    izip!(
                        Coordinate::<Base>::all(),
                        group_candidate_indexes.row_major_blocks,
                    )
                    .filter_map(
                        move |(block_i, block): (Coordinate<Base>, Candidates<Base>)| {
                            let Some(segment_index) = block.block_segmentation() else {
                                return None;
                            };
                            let (block_row, block_column) = block_i.to_block_row_and_column();
                            let rows_i = (block_row, segment_index).into();
                            let row = group_candidate_indexes.rows.get(rows_i);
                            let None = row.block_segmentation() else { return None; };

                            // Found group intersection with an effect.

                            let mut deduction = Deduction::new();
                            let reason = Reason::candidate(candidate);
                            for row_major_index in block
                                .intersection(Candidates::block_segmentation_mask(segment_index))
                                .into_iter()
                                .map(Coordinate::from)
                            {
                                deduction
                                    .reasons
                                    .insert(
                                        Position::with_block_and_row_major_index((
                                            block_i,
                                            row_major_index,
                                        )),
                                        reason,
                                    )
                                    .unwrap();
                            }

                            let action = Action::delete_candidate(candidate);
                            for row_i in row
                                .without(Candidates::block_segmentation_mask(block_column))
                                .into_iter()
                                .map(Coordinate::from)
                            {
                                deduction
                                    .actions
                                    .insert((rows_i, row_i).into(), action)
                                    .unwrap();
                            }

                            Some(deduction)
                        },
                    )
                })
                .collect(),
        )
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

        assert_eq!(
            deductions,
            vec![Deduction::try_from_iters(
                vec![
                    (
                        (3, 2).try_into().unwrap(),
                        Reason::candidate(1.try_into().unwrap())
                    ),
                    (
                        (3, 3).try_into().unwrap(),
                        Reason::candidate(1.try_into().unwrap())
                    )
                ]
                .into_iter(),
                vec![
                    (
                        (3, 0).try_into().unwrap(),
                        Action::delete_candidate(1.try_into().unwrap())
                    ),
                    (
                        (3, 1).try_into().unwrap(),
                        Action::delete_candidate(1.try_into().unwrap())
                    ),
                ]
                .into_iter()
            )
            .unwrap()]
            .into_iter()
            .collect()
        );
    }
}
