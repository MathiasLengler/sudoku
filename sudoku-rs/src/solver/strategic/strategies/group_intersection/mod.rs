use itertools::{chain, izip};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::Value;
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GroupIntersectionBlockToAxis;
impl Strategy for GroupIntersectionBlockToAxis {
    fn name(self) -> &'static str {
        "GroupIntersectionBlockToAxis"
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        GroupIntersection(GroupIntersectionTypeFilter::BlockToAxis).execute(grid)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GroupIntersectionAxisToBlock;
impl Strategy for GroupIntersectionAxisToBlock {
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        GroupIntersection(GroupIntersectionTypeFilter::AxisToBlock).execute(grid)
    }

    fn name(self) -> &'static str {
        "GroupIntersectionAxisToBlock"
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GroupIntersectionBoth;
impl Strategy for GroupIntersectionBoth {
    fn name(self) -> &'static str {
        "GroupIntersectionBoth"
    }
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        GroupIntersection(GroupIntersectionTypeFilter::Both).execute(grid)
    }
}

/// An implementation of the group intersection / ["intersection removal"](https://www.sudokuwiki.org/Intersection_Removal) strategy.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct GroupIntersection(GroupIntersectionTypeFilter);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GroupIntersectionTypeFilter {
    /// "Pointing Pairs, Pointing Triples"
    /// Find segmented block which reduces the axis.
    BlockToAxis,
    /// "Box/Line Reduction" (segmented axis, reducing block)
    /// Find segmented axis which reduces the block
    AxisToBlock,
    /// Find reducing intersections irrespective of the inference direction.
    Both,
}

enum GroupIntersectionType {
    /// "Pointing Pairs, Pointing Triples"
    /// Find segmented block which reduces the axis.
    BlockToAxis,
    /// "Box/Line Reduction" (segmented axis, reducing block)
    /// Find segmented axis which reduces the block
    AxisToBlock,
}

impl Strategy for GroupIntersection {
    fn name(self) -> &'static str {
        "GroupIntersection"
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        let candidate_to_group_candidate_indexes = GroupCandidateIndexes::with_grid(grid);

        Ok(
            izip!(Value::<Base>::all(), &candidate_to_group_candidate_indexes)
                .flat_map(|(candidate, group_candidate_indexes)| {
                    chain!(
                        BlockSegment::<Base>::all_with_orientation(CellOrder::RowMajor).filter_map(
                            move |block_segment| {
                                group_candidate_indexes.evaluate(candidate, block_segment)
                            }
                        ),
                        BlockSegment::<Base>::all_with_orientation(CellOrder::ColumnMajor)
                            .filter_map(move |block_segment| {
                                group_candidate_indexes.evaluate(candidate, block_segment)
                            }),
                    )
                })
                .collect(),
        )
    }
}

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

    fn evaluate(
        &self,
        candidate: Value<Base>,
        block_segment: BlockSegment<Base>,
    ) -> Option<Deduction<Base>> {
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
                // Found: "Box/Line Reduction" (segmented axis, reducing block)

                let mut deduction = Deduction::new();
                let reason = Reason::candidate(candidate);
                for axis_index in axis_candidate_positions
                    .intersection(block_segment.axis_mask())
                    .into_iter()
                    .map(Coordinate::from)
                {
                    deduction
                        .reasons
                        .insert(block_segment.axis_position(axis_index), reason)
                        .unwrap();
                }

                let action = Action::delete_candidate(candidate);
                for block_index in block_candidate_positions
                    .without(block_segment.block_mask())
                    .into_iter()
                    .map(Coordinate::from)
                {
                    deduction
                        .actions
                        .insert(block_segment.block_position(block_index), action)
                        .unwrap();
                }

                Some(deduction)
            }
            (None, Some(block_segment_index))
                if block_segment_index == block_segment.block_segment_index() =>
            {
                // Found: "Pointing Pairs, Pointing Triples" (segmented block, reducing axis)
                let mut deduction = Deduction::new();
                let reason = Reason::candidate(candidate);
                for block_index in block_candidate_positions
                    .intersection(block_segment.block_mask())
                    .into_iter()
                    .map(Coordinate::from)
                {
                    deduction
                        .reasons
                        .insert(block_segment.block_position(block_index), reason)
                        .unwrap();
                }

                let action = Action::delete_candidate(candidate);

                for axis_index in axis_candidate_positions
                    .without(block_segment.axis_mask())
                    .into_iter()
                    .map(Coordinate::from)
                {
                    deduction
                        .actions
                        .insert(block_segment.axis_position(axis_index), action)
                        .unwrap();
                }

                Some(deduction)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    #[test]
    fn test_execute_base_2() {
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

        let deductions = GroupIntersection(GroupIntersectionTypeFilter::Both)
            .execute(&grid)
            .unwrap();

        println!("Deductions:\n{deductions}");

        assert_eq!(
            deductions,
            vec![
                Deduction::try_from_iters(
                    vec![
                        ((3, 0), Action::delete_candidate(1.try_into().unwrap())),
                        ((3, 1), Action::delete_candidate(1.try_into().unwrap())),
                    ],
                    vec![
                        ((3, 2), Reason::candidate(1.try_into().unwrap())),
                        ((3, 3), Reason::candidate(1.try_into().unwrap()))
                    ],
                )
                .unwrap(),
                Deduction::try_from_iters(
                    vec![
                        ((3, 0), Action::delete_candidate(1.try_into().unwrap())),
                        ((3, 1), Action::delete_candidate(1.try_into().unwrap())),
                    ],
                    vec![
                        ((2, 0), Reason::candidate(1.try_into().unwrap())),
                        ((2, 1), Reason::candidate(1.try_into().unwrap()))
                    ],
                )
                .unwrap(),
            ]
            .into_iter()
            .collect()
        );

        let merged_deductions = deductions.merge_deductions_by_actions().unwrap();
        println!("Merged deductions:\n{merged_deductions}");
    }

    #[test]
    fn test_execute_base_3() {}
}
