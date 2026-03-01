use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Coordinate;
use crate::position::Position;
use crate::solver::strategic::deduction::Action;
use crate::solver::strategic::deduction::Deduction;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::deduction::Reason;
use crate::solver::strategic::group_candidate_availability::{
    Axis, GroupCandidateAvailability, StrategicGroupAvailability,
};
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;
use std::collections::BTreeMap;

/*
Logic:

For each candidate:

build subset of `GroupCandidateIndexes`:
```
rows: CandidatesGroup<Base>,
columns: CandidatesGroup<Base>,
```
With this precomputed data stucture, we can answer quickly:
- In this row or column, in which position (Coordinate) is the candidate set?
- How many candidates are set in this row or column? (bit count)
- Are the candidates of two rows or two columns equal? (integer comparison)

For X-Wing, we are intereseted in rows or columns with exactly two candidates set ("locked pair")
More precisely, for a given candidate, we want to find two locked pairs in the same column/row, where the candidate is also set in the orthoganal direction.

Starting with rows, then inversely for columns.

Filter rows with a locked pair (candidates count 2)
Count the locked pair patterns (based on Candidates equality)
evaluate counts:
if 1: nothing
if 2: X-Wing candidate (could have no effect)
if >2: Sudoku is unsolvable: the 8 shape resolves to a > or < shape, in which one column always contains a duplicate value.

A X-Wing candidate is identified by:
2 distinct row coordinates
2 distinct column coordinates
(sides of a square)

Since we arrived at the X-Wing candidate via rows, we now look a the affected columns for eliminations.
Get both columns, count candidates:
if 1: panic (we expect at least 2)
if 2: only the X-Wing candidate itself, nothing to eliminate.
if >2: X-Wing found.

Candidates to delete: `column_candidates without X-Wing candidates (2 row coordinates)`

Deduction:
`Action::DeleteCandidates(Candidates::with_single(candidate))` - positions: affected columns, excluding X-Wing rows
`Reason::Candidates(Candidates::with_single(candidate))` - the four positions of the X-Wing

Repeat the same logic, starting with columns.
Repeat for all candidates.

Evaluate: Generalising X-Wing
  "X-Wings containing boxes are the inverse of the Intersection Removal strategies"
There is a reason why the logic seems simmilar to `GroupIntersection`.
Could it make sense to generalise both strategies into a single implementation, parameterised by the pattern to search for?
    Analog to the LockedSets strategy?
First implement the basic X-Wing strategy as is, then consider generalisation. Otherwise this could get too complex.
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct XWing;

impl Strategy for XWing {
    fn name(self) -> &'static str {
        "XWing"
    }

    fn score(self) -> StrategyScore {
        200
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        // Fall back to building the availability from scratch when not provided
        let availability = StrategicGroupAvailability::from_grid(grid);
        self.execute_with_availability(grid, &availability)
    }

    fn execute_with_availability<Base: SudokuBase>(
        self,
        _grid: &Grid<Base>,
        group_availability: &StrategicGroupAvailability<Base>,
    ) -> Result<Deductions<Base>> {
        Ok(group_availability
            .iter()
            .flat_map(|(candidate, candidate_availability)| {
                find_x_wings(candidate, candidate_availability)
            })
            .collect())
    }
}

fn find_x_wings<Base: SudokuBase>(
    candidate: Value<Base>,
    candidate_availability: &GroupCandidateAvailability<Base>,
) -> impl Iterator<Item = Deduction<Base>> + '_ {
    find_x_wings_axis(candidate, candidate_availability, Axis::Row).chain(find_x_wings_axis(
        candidate,
        candidate_availability,
        Axis::Column,
    ))
}

fn find_x_wings_axis<Base: SudokuBase>(
    candidate: Value<Base>,
    candidate_availability: &GroupCandidateAvailability<Base>,
    axis: Axis,
) -> impl Iterator<Item = Deduction<Base>> + '_ {
    let opposite_axis = axis.other();

    let locked_pairs: Vec<_> = candidate_availability
        .axis(axis)
        .iter_enumerate()
        .filter(|(_, candidates)| candidates.count() == 2)
        .collect();

    let mut locked_pair_pattern_to_axis_coordinates: BTreeMap<Candidates<_>, Candidates<_>> =
        BTreeMap::new();

    for (axis_coordinate, locked_pair_pattern) in locked_pairs {
        locked_pair_pattern_to_axis_coordinates
            .entry(locked_pair_pattern)
            .and_modify(|axis_coordinates| axis_coordinates.set(axis_coordinate, true))
            .or_insert(Candidates::with_single(axis_coordinate.into()));
    }

    locked_pair_pattern_to_axis_coordinates
        .into_iter()
        .filter_map(move |(locked_pair_pattern, axis_coordinates)| {
            find_x_wing_candidate(locked_pair_pattern, axis_coordinates, axis, candidate)
        })
        .filter_map(move |x_wing_candidate| {
            let candiates_positions_to_delete: Vec<_> = x_wing_candidate
                .axis_coordinates(opposite_axis)
                .into_iter()
                .filter_map(|other_axis_coordinate| {
                    let axis_coordinates = candidate_availability
                        .axis(opposite_axis)
                        .get(other_axis_coordinate.into());

                    let axis_coordinates_to_delete =
                        axis_coordinates.without(x_wing_candidate.axis_coordinates(axis));

                    if axis_coordinates_to_delete.is_empty() {
                        None
                    } else {
                        Some(axis_coordinates_to_delete.into_iter().map(
                            move |axis_coordinate_to_delete| {
                                axis.coordinates_to_pos(
                                    Coordinate::from(axis_coordinate_to_delete),
                                    Coordinate::from(other_axis_coordinate),
                                )
                            },
                        ))
                    }
                })
                .flatten()
                .collect();

            if candiates_positions_to_delete.is_empty() {
                return None;
            }

            Some(
                Deduction::try_from_iters(
                    candiates_positions_to_delete
                        .into_iter()
                        .map(|candiates_position_to_delete| {
                            (
                                candiates_position_to_delete,
                                Action::delete_candidate(x_wing_candidate.candidate),
                            )
                        }),
                    x_wing_candidate
                        .to_positions()
                        .map(|pos| (pos, Reason::candidate(x_wing_candidate.candidate))),
                )
                .unwrap(),
            )
        })
}

fn find_x_wing_candidate<Base: SudokuBase>(
    locked_pair_pattern: Candidates<Base>,
    axis_coordinates: Candidates<Base>,
    axis: Axis,
    candidate: Value<Base>,
) -> Option<XWingPattern<Base>> {
    match axis_coordinates.count() {
        0 => {
            panic!("Expected at least one coordinate for pattern {locked_pair_pattern}",)
        }
        1 => None,
        2 => {
            // X-Wing candidate found
            assert!(
                locked_pair_pattern.count() == 2,
                "Expected locked pair pattern to have exactly two candidates set, got {locked_pair_pattern}"
            );
            Some(match axis {
                Axis::Row => XWingPattern::new(candidate, axis_coordinates, locked_pair_pattern),
                Axis::Column => XWingPattern::new(candidate, locked_pair_pattern, axis_coordinates),
            })
        }
        _ => {
            panic!(
                "Sudoku is unsolvable: conflicting axis coordinates {axis_coordinates} for locked pair pattern {locked_pair_pattern}"
            )
        }
    }
}

/// A detected X-Wing pattern in a sudoku grid.
///
/// It consists of a candidate in 4 cells, forming a rectangle. In one axis, the candidates of the X-Wing are the only ones remaining.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct XWingPattern<Base: SudokuBase> {
    /// The candidate which forms the X-Wing.
    candidate: Value<Base>,
    /// The two row coordinates of the X-Wing rectangle.
    row_coordinates: Candidates<Base>,
    /// The two column coordinates of the X-Wing rectangle.
    column_coordinates: Candidates<Base>,
}

impl<Base: SudokuBase> XWingPattern<Base> {
    fn new(
        candidate: Value<Base>,
        row_coordinates: Candidates<Base>,
        column_coordinates: Candidates<Base>,
    ) -> Self {
        debug_assert_eq!(row_coordinates.count(), 2);
        debug_assert_eq!(column_coordinates.count(), 2);
        Self {
            candidate,
            row_coordinates,
            column_coordinates,
        }
    }

    fn axis_coordinates(&self, axis: Axis) -> Candidates<Base> {
        match axis {
            Axis::Row => self.row_coordinates,
            Axis::Column => self.column_coordinates,
        }
    }

    /// Iterate over the four positions forming the X-Wing rectangle.
    fn to_positions(self) -> impl Iterator<Item = Position<Base>> {
        self.row_coordinates
            .into_iter()
            .flat_map(move |row_coordinate| {
                self.column_coordinates
                    .into_iter()
                    .map(move |column_coordinate| {
                        Position::from((
                            Coordinate::from(row_coordinate),
                            Coordinate::from(column_coordinate),
                        ))
                    })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::consts::*,
        cell::Cell,
        solver::strategic::strategies::test_util::{
            assert_deductions, assert_deductions_with_grid, strategy_snapshot_tests,
        },
    };
    use indoc::indoc;
    use std::fmt::Debug;

    fn x_wing_deduction<Base: SudokuBase>(
        candidate: impl TryInto<Value<Base>, Error: Debug>,
        positions_to_delete: impl IntoIterator<Item = (u8, u8)>,
        x_wing_positions: impl IntoIterator<Item = (u8, u8)>,
    ) -> Deduction<Base> {
        let candidate = candidate.try_into().unwrap();
        Deduction::try_from_iters(
            positions_to_delete
                .into_iter()
                .map(|pos| (pos, Action::delete_candidate(candidate))),
            x_wing_positions
                .into_iter()
                .map(|pos| (pos, Reason::candidate(candidate))),
        )
        .unwrap()
    }

    mod synthetic {
        use super::*;

        #[test]
        fn test_synthetic_base_2_row_to_column() {
            let mut grid: Grid<Base2> = indoc! {"
            0100
            0101
            0101
            0001"
            }
            .parse()
            .unwrap();
            for pos in grid.all_value_positions() {
                grid[pos] = Cell::with_candidates(grid[pos].to_candidates());
            }

            let deductions = XWing.execute(&grid).unwrap();

            let expected_deductions = x_wing_deduction(
                1,
                vec![(0, 1), (3, 3)],
                vec![(1, 1), (1, 3), (2, 1), (2, 3)],
            )
            .into();

            assert_deductions(&deductions, &expected_deductions);
        }

        #[test]
        fn test_base_2_column_to_row() {
            let mut grid: Grid<Base2> = indoc! {"
            1000
            1101
            0111
            0010"
            }
            .parse()
            .unwrap();
            for pos in grid.all_value_positions() {
                grid[pos] = Cell::with_candidates(grid[pos].to_candidates());
            }

            let deductions = XWing.execute(&grid).unwrap();

            let expected_deductions = x_wing_deduction(
                1,
                vec![(1, 0), (2, 2)],
                vec![(1, 1), (1, 3), (2, 1), (2, 3)],
            )
            .into();

            assert_deductions(&deductions, &expected_deductions);
        }
    }

    mod sudokuwiki_examples {
        use super::*;

        #[test]
        fn test_example_1() {
            // Reference: https://www.sudokuwiki.org/X_Wing_Strategy#:~:text=at%20a%20time.-,X%2DWing%20example%201,-%3A%20Load%20Example
            let mut grid: Grid<Base3> = "S9B015y2e685w68050609040i022e0e0f0a2e085y050f0a5u090b042e2u2e0i06042c0810012q0f0dd0015w9i102e020a089e03050f9e0d5y042e05d0609i010f095y0e5y0f0a045y0206020166cy669id205".parse().unwrap();

            let deductions = XWing.execute(&grid).unwrap();

            let expected_deductions = x_wing_deduction(
                7,
                vec![
                    // Left column
                    (0, 3),
                    (4, 3),
                    (7, 3),
                    (8, 3),
                    // Right column
                    (7, 7),
                    (8, 7),
                ],
                vec![(1, 3), (1, 7), (5, 3), (5, 7)],
            )
            .into();

            assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
        }

        #[test]
        fn test_example_2() {
            // Reference: https://www.sudokuwiki.org/X_Wing_Strategy#:~:text=an%20example%20next.-,X%2DWing%20example%202,-%3A%20Load%20Example
            let mut grid: Grid<Base3> = "S9B4n144p5i6q7a360i0407064a0i014a0o050o1a091a263e023608011q078w580556bk01bc2714290758094w0o4816088c0s030a8c060g02045i014y5iba07c652016u54096u48040509127a5s707i0a0o54".parse().unwrap();

            let deductions = XWing.execute(&grid).unwrap();

            let expected_deductions = x_wing_deduction(
                2,
                vec![
                    // Top row
                    (4, 1),
                    (4, 2),
                    (4, 6),
                    (4, 8),
                    // Bottom row
                    (8, 3),
                    (8, 8),
                ],
                vec![(4, 4), (4, 7), (8, 4), (8, 7)],
            )
            .into();

            assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
        }

        #[test]
        fn test_example_3() {
            // Reference: https://www.sudokuwiki.org/X_Wing_Strategy#:~:text=yellow%20highlighted%20numbers.-,X%2DWing%20example%203,-%3A%20Load%20Example
            let mut grid: Grid<Base3> = "S9B4n0b4n5i6q7a360i0407064a0i014a0o050o1a091a263e023608011q077o580556bk01bc2712270758094u0o4616087o0s030a8c060g02045i014y5iba07c652016u54096u48040509127a5q707i0a0o52".parse().unwrap();

            let deductions = XWing.execute(&grid).unwrap();

            let expected_deductions = x_wing_deduction(
                3,
                vec![
                    // Top row
                    (4, 0),
                    (4, 2),
                    (4, 6),
                    (4, 8),
                    // Bottom row
                    (8, 2),
                    (8, 3),
                    (8, 5),
                    (8, 8),
                ],
                vec![(4, 1), (4, 7), (8, 1), (8, 7)],
            )
            .into();

            assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
        }
    }

    strategy_snapshot_tests!(XWing);
}
