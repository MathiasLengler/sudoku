use itertools::izip;

use crate::base::SudokuBase;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::{BlockSegment, CellOrder, Coordinate, Position};
use crate::solver::backtracking_bitset::group_availability::CandidatesGroup;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};
use crate::solver::strategic::strategies::Strategy;

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
    fn name(self) -> &'static str {
        "GroupIntersectionAxisToBlock"
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        GroupIntersection(GroupIntersectionTypeFilter::AxisToBlock).execute(grid)
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

/// An implementation of the group intersection / [intersection removal](https://www.sudokuwiki.org/Intersection_Removal) strategy.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct GroupIntersection(GroupIntersectionTypeFilter);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GroupIntersectionTypeFilter {
    /// "Pointing Pairs, Pointing Triples"
    ///
    /// Find segmented block which reduces the axis.
    BlockToAxis,
    /// "Box/Line Reduction"
    ///
    /// Find segmented axis which reduces the block.
    AxisToBlock,
    /// Find reducing intersections irrespective of the inference direction.
    Both,
}

impl GroupIntersectionTypeFilter {
    fn includes(self, group_intersection_type: GroupIntersectionType) -> bool {
        match self {
            GroupIntersectionTypeFilter::BlockToAxis => {
                matches!(group_intersection_type, GroupIntersectionType::BlockToAxis)
            }
            GroupIntersectionTypeFilter::AxisToBlock => {
                matches!(group_intersection_type, GroupIntersectionType::AxisToBlock)
            }
            GroupIntersectionTypeFilter::Both => true,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GroupIntersectionType {
    /// "Pointing Pairs, Pointing Triples"
    ///
    /// Find segmented block which reduces the axis.
    BlockToAxis,
    /// "Box/Line Reduction"
    ///
    /// Find segmented axis which reduces the block
    AxisToBlock,
}

impl Strategy for GroupIntersection {
    fn name(self) -> &'static str {
        "GroupIntersection"
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        let group_intersection_type_filter = self.0;

        let candidate_to_group_candidate_indexes = GroupCandidateIndexes::with_grid(grid);

        Ok(
            izip!(Value::<Base>::all(), candidate_to_group_candidate_indexes)
                .flat_map(|(candidate, group_candidate_indexes)| {
                    BlockSegment::<Base>::all().filter_map(move |block_segment| {
                        group_candidate_indexes.evaluate(
                            candidate,
                            block_segment,
                            group_intersection_type_filter,
                        )
                    })
                })
                .collect(),
        )
    }
}

/// For a single candidate, where in each group is this candidate set?
#[derive(Debug, Clone, Default)]
struct GroupCandidateIndexes<Base: SudokuBase> {
    /// intersects with `self.row_major_blocks`
    rows: CandidatesGroup<Base>,
    /// intersects with `self.column_major_blocks`
    columns: CandidatesGroup<Base>,
    /// intersects with `self.rows`
    row_major_blocks: CandidatesGroup<Base>,
    /// intersects with `self.columns`
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
        group_intersection_type_filter: GroupIntersectionTypeFilter,
    ) -> Option<Deduction<Base>> {
        let (axis_candidate_positions, block_candidate_positions) = match block_segment
            .orientation()
        {
            CellOrder::RowMajor => {
                let row_candidate_positions = self.rows.get(block_segment.axis());
                let block_candidate_positions = self.row_major_blocks.get(block_segment.block());

                (row_candidate_positions, block_candidate_positions)
            }
            CellOrder::ColumnMajor => {
                let column_candidate_positions = self.columns.get(block_segment.axis());
                let block_candidate_positions = self.column_major_blocks.get(block_segment.block());
                (column_candidate_positions, block_candidate_positions)
            }
        };

        let group_intersection_type = match (
            axis_candidate_positions.block_segmentation(),
            block_candidate_positions.block_segmentation(),
        ) {
            (Some(axis_segment_index), None)
                if axis_segment_index == block_segment.axis_segment_index() =>
            {
                GroupIntersectionType::AxisToBlock
            }
            (None, Some(block_segment_index))
                if block_segment_index == block_segment.block_segment_index() =>
            {
                GroupIntersectionType::BlockToAxis
            }
            _ => {
                return None;
            }
        };

        if !group_intersection_type_filter.includes(group_intersection_type) {
            return None;
        }

        let action = Action::delete_candidate(candidate);
        let reason = Reason::candidate(candidate);
        Some(match group_intersection_type {
            GroupIntersectionType::BlockToAxis => Deduction::try_from_iters(
                axis_candidate_positions
                    .without(block_segment.axis_mask())
                    .into_iter()
                    .map(Coordinate::from)
                    .map(|axis_index| (block_segment.axis_position(axis_index), action)),
                block_candidate_positions
                    .intersection(block_segment.block_mask())
                    .into_iter()
                    .map(Coordinate::from)
                    .map(|block_index| (block_segment.block_position(block_index), reason)),
            )
            .unwrap(),
            GroupIntersectionType::AxisToBlock => Deduction::try_from_iters(
                block_candidate_positions
                    .without(block_segment.block_mask())
                    .into_iter()
                    .map(Coordinate::from)
                    .map(|block_index| (block_segment.block_position(block_index), action)),
                axis_candidate_positions
                    .intersection(block_segment.axis_mask())
                    .into_iter()
                    .map(Coordinate::from)
                    .map(|axis_index| (block_segment.axis_position(axis_index), reason)),
            )
            .unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    fn expected_deduction<Base: SudokuBase>(
        candidate: u8,
        action_positions: Vec<(u8, u8)>,
        reason_positions: Vec<(u8, u8)>,
    ) -> Deduction<Base> {
        let candidate: Value<Base> = candidate.try_into().unwrap();
        Deduction::try_from_iters(
            action_positions
                .into_iter()
                .map(|pos| (pos, Action::delete_candidate(candidate))),
            reason_positions
                .into_iter()
                .map(|pos| (pos, Reason::candidate(candidate))),
        )
        .unwrap()
    }

    mod execute {
        use super::*;

        mod base_2 {
            use super::*;

            #[test]
            fn test() {
                let grid: Grid<Base2> = "╔═══════════╦═══════════╗
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
╚═══════════╩═══════════╝"
                    .parse()
                    .unwrap();

                let expected_deduction_block_to_row = Deduction::try_from_iters(
                    vec![
                        ((3, 0), Action::delete_candidate(1.try_into().unwrap())),
                        ((3, 1), Action::delete_candidate(1.try_into().unwrap())),
                    ],
                    vec![
                        ((3, 2), Reason::candidate(1.try_into().unwrap())),
                        ((3, 3), Reason::candidate(1.try_into().unwrap())),
                    ],
                )
                .unwrap();
                let expected_deduction_row_to_block = Deduction::try_from_iters(
                    vec![
                        ((3, 0), Action::delete_candidate(1.try_into().unwrap())),
                        ((3, 1), Action::delete_candidate(1.try_into().unwrap())),
                    ],
                    vec![
                        ((2, 0), Reason::candidate(1.try_into().unwrap())),
                        ((2, 1), Reason::candidate(1.try_into().unwrap())),
                    ],
                )
                .unwrap();

                let deductions = GroupIntersection(GroupIntersectionTypeFilter::Both)
                    .execute(&grid)
                    .unwrap();
                assert_eq!(
                    deductions,
                    vec![
                        expected_deduction_block_to_row.clone(),
                        expected_deduction_row_to_block.clone()
                    ]
                    .into_iter()
                    .collect()
                );

                let deductions = GroupIntersection(GroupIntersectionTypeFilter::BlockToAxis)
                    .execute(&grid)
                    .unwrap();
                assert_eq!(
                    deductions,
                    vec![expected_deduction_block_to_row].into_iter().collect()
                );

                let deductions = GroupIntersection(GroupIntersectionTypeFilter::AxisToBlock)
                    .execute(&grid)
                    .unwrap();
                assert_eq!(
                    deductions,
                    vec![expected_deduction_row_to_block].into_iter().collect()
                );
            }
        }

        mod base_3 {
            use super::*;

            mod block_to_axis {
                use super::*;

                #[test]
                fn test_pointing_pairs_example_1() {
                    // Reference: https://www.sudokuwiki.org/Intersection_Removal#:~:text=Pointing%20Pairs%20%3A%20Load%20Example
                    let grid: Grid<Base3> = "9k0341g11k09218g8k3s1s28568150gagigug18sa8062k2g118i419041059003i00h09i09ap8o80hj005o241q282210h0941o00511o241os0384gkogo82111akokq0500950o2oioi8ooo1121gg034105oo".parse().unwrap();

                    let deductions = GroupIntersection(GroupIntersectionTypeFilter::BlockToAxis)
                        .execute(&grid)
                        .unwrap();

                    let expected_deductions = vec![
                        expected_deduction(3, vec![(1, 0), (1, 1), (1, 2)], vec![(1, 6), (1, 8)]),
                        expected_deduction(6, vec![(2, 2)], vec![(2, 4), (2, 5)]),
                        expected_deduction(9, vec![(4, 4), (4, 6), (4, 8)], vec![(4, 1), (4, 2)]),
                        expected_deduction(2, vec![(6, 1)], vec![(6, 3), (6, 4)]),
                        expected_deduction(8, vec![(6, 1), (6, 6)], vec![(6, 3), (6, 5)]),
                    ]
                    .into_iter()
                    .collect();

                    deductions.validate(&grid).unwrap();

                    assert_eq!(
                        deductions, expected_deductions,
                        "{deductions}\n!==\n{expected_deductions}"
                    );
                }

                #[test]
                fn test_pointing_pairs_example_2() {
                    // Reference: https://www.sudokuwiki.org/Intersection_Removal#:~:text=Pointing%20Pairs%20Example%202
                    let grid: Grid<Base3> = "s00905cgdg2103pgc00h03r0ccd85cmcpcece0c0b0g1do036s9sec11c48222g1482c8c0ho421og8o9o1ogc410209sgoi22054gi0o011i6gkiq116q814s0s4ca48kao4s6o4s1003g10610410s0qg081210c".parse().unwrap();

                    let deductions = GroupIntersection(GroupIntersectionTypeFilter::BlockToAxis)
                        .execute(&grid)
                        .unwrap();

                    let expected_deductions = vec![
                        expected_deduction(8, vec![(0, 7), (1, 7), (2, 7)], vec![(3, 7), (5, 7)]),
                        expected_deduction(7, vec![(1, 5), (7, 5)], vec![(3, 5), (5, 5)]),
                        expected_deduction(2, vec![(1, 6), (1, 7), (1, 8)], vec![(1, 3), (1, 5)]),
                        expected_deduction(6, vec![(1, 6), (2, 6)], vec![(3, 6), (5, 6)]),
                        expected_deduction(7, vec![(2, 1)], vec![(3, 1), (5, 1)]),
                        expected_deduction(8, vec![(4, 0), (4, 2)], vec![(4, 3), (4, 4)]),
                        expected_deduction(4, vec![(6, 1), (6, 2), (6, 4)], vec![(6, 6), (6, 7)]),
                        expected_deduction(1, vec![(6, 2)], vec![(3, 2), (5, 2)]),
                        expected_deduction(7, vec![(6, 4)], vec![(6, 6), (6, 8)]),
                    ]
                    .into_iter()
                    .collect();

                    deductions.validate(&grid).unwrap();

                    assert_eq!(
                        deductions, expected_deductions,
                        "{deductions}\n!==\n{expected_deductions}"
                    );
                }
                #[test]
                fn test_pointing_tripple_example() {
                    // Reference: https://www.sudokuwiki.org/Intersection_Removal#:~:text=Pointing%20Triple%20%3A%20Load%20Example
                    let grid: Grid<Base3> = "g1094i4g1182ek2m66054i4i210982cgg111811121kgkg054o0q4a2gg4090381ig1141246i6i114o056og1812a6i81g4kokg112s2u2e6o6k4k814g4o0311g111m0810503k868280h4qkiki1121ko4c0c81".parse().unwrap();

                    let deductions = GroupIntersection(GroupIntersectionTypeFilter::BlockToAxis)
                        .execute(&grid)
                        .unwrap();

                    let expected_deductions = vec![
                        expected_deduction(7, vec![(0, 6), (1, 6), (2, 6)], vec![(7, 6), (8, 6)]),
                        expected_deduction(9, vec![(3, 5)], vec![(7, 5), (8, 5)]),
                        expected_deduction(3, vec![(4, 5)], vec![(6, 5), (7, 5), (8, 5)]),
                        expected_deduction(4, vec![(5, 0), (5, 3), (5, 4)], vec![(5, 6), (5, 7)]),
                        expected_deduction(6, vec![(7, 1)], vec![(7, 6), (7, 7)]),
                    ]
                    .into_iter()
                    .collect();

                    deductions.validate(&grid).unwrap();

                    assert_eq!(
                        deductions, expected_deductions,
                        "{deductions}\n!==\n{expected_deductions}"
                    );
                }
            }

            mod axis_to_block {
                use super::*;

                #[test]
                fn test_example_1() {
                    // Reference: https://www.sudokuwiki.org/Intersection_Removal#:~:text=Box%20Line%20Reduction-,Box/Line%20Reduction%20%3A%20Load%20Example,-or%20%3A%20From
                    let grid: Grid<Base3> = "1g03211khk4181gg091og11c813s3o4m4g5i81411c1shs030k21hg460h8156763009k0k22111424q4qg14i81054609g14mcm8g21114i5a215ag1d2904g05cg5281525i5i05g10921g1050h21c8881103c0".parse().unwrap();

                    let deductions = GroupIntersection(GroupIntersectionTypeFilter::AxisToBlock)
                        .execute(&grid)
                        .unwrap();

                    let expected_deductions = vec![
                        expected_deduction(2, vec![(1, 4), (2, 3), (2, 4)], vec![(0, 3), (0, 4)]),
                        expected_deduction(
                            4,
                            vec![(1, 6), (1, 8), (2, 6), (2, 8)],
                            vec![(0, 7), (1, 7)],
                        ),
                    ]
                    .into_iter()
                    .collect();

                    deductions.validate(&grid).unwrap();

                    assert_eq!(
                        deductions, expected_deductions,
                        "{deductions}\n!==\n{expected_deductions}"
                    );
                }
                #[test]
                fn test_example_2() {
                    // Reference: https://www.sudokuwiki.org/Intersection_Removal#:~:text=in%20C7.-,Triple%20BLR,-%3A%20Load%20Example
                    let grid: Grid<Base3> = "a005a0g10h09410311g10a0hd24652210c8441110aa22622o80ho4116ama0h81m2g2k4m405e2u262m2m20h11090h62m2091105o2k0u0280h0570m8n0g881038aca1142ka0h0521k02ag16a056a8111480h".parse().unwrap();

                    println!("{grid}");

                    let deductions = GroupIntersection(GroupIntersectionTypeFilter::AxisToBlock)
                        .execute(&grid)
                        .unwrap();

                    let expected_deductions = vec![
                        expected_deduction(
                            6,
                            vec![(3, 2), (4, 2), (5, 2)],
                            vec![(3, 1), (4, 1), (5, 1)],
                        ),
                        expected_deduction(
                            9,
                            vec![(3, 6), (3, 8), (5, 6), (5, 8)],
                            vec![(3, 7), (5, 7)],
                        ),
                        expected_deduction(1, vec![(7, 1), (8, 2)], vec![(7, 0), (8, 0)]),
                        expected_deduction(3, vec![(7, 1), (8, 2)], vec![(6, 0), (7, 0), (8, 0)]),
                        expected_deduction(
                            7,
                            vec![(7, 3), (7, 4), (8, 4)],
                            vec![(6, 3), (6, 4), (6, 5)],
                        ),
                    ]
                    .into_iter()
                    .collect();

                    deductions.validate(&grid).unwrap();

                    assert_eq!(
                        deductions, expected_deductions,
                        "{deductions}\n!==\n{expected_deductions}"
                    );
                }
            }
        }
    }
}
