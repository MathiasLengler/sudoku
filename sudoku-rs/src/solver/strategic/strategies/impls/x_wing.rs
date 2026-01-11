use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::grid::group::CandidatesGroup;
use crate::position::Coordinate;
use crate::position::Position;
use crate::solver::strategic::deduction::Action;
use crate::solver::strategic::deduction::Deduction;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::deduction::Reason;
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;
use itertools::izip;
use std::collections::BTreeMap;

// TODO: implement: https://www.sudokuwiki.org/X_Wing_Strategy

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
        let candidate_to_group_candidate_indexes = GroupCandidateIndexes::with_grid(grid);

        Ok(izip!(
            Value::<Base>::all(),
            candidate_to_group_candidate_indexes.iter()
        )
        .flat_map(|(candidate, group_candidate_indexes)| {
            find_x_wings(candidate, group_candidate_indexes)
        })
        .collect())
    }
}

fn find_x_wings<Base: SudokuBase>(
    candidate: Value<Base>,
    group_candidate_indexes: &GroupCandidateIndexes<Base>,
) -> impl Iterator<Item = Deduction<Base>> + '_ {
    find_x_wings_axis(candidate, group_candidate_indexes, Axis::Row).chain(find_x_wings_axis(
        candidate,
        group_candidate_indexes,
        Axis::Column,
    ))
}

fn find_x_wings_axis<Base: SudokuBase>(
    candidate: Value<Base>,
    group_candidate_indexes: &GroupCandidateIndexes<Base>,
    axis: Axis,
) -> impl Iterator<Item = Deduction<Base>> + '_ {
    let opposite_axis = axis.other();

    let locked_pairs: Vec<_> = group_candidate_indexes
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
                    let axis_coordinates = group_candidate_indexes
                        .axis(opposite_axis)
                        .get(other_axis_coordinate.into());

                    let axis_coordinates_to_delete =
                        axis_coordinates.without(x_wing_candidate.axis_coordinates(axis));

                    if axis_coordinates_to_delete.is_empty() {
                        None
                    } else {
                        Some(axis_coordinates_to_delete.into_iter().map(
                            move |axis_coordinate_to_delete| {
                                Position::from((
                                    Coordinate::from(axis_coordinate_to_delete),
                                    Coordinate::from(other_axis_coordinate),
                                ))
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

/// For a single candidate, where in each group is this candidate set?
#[derive(Debug, Clone, Default)]
struct GroupCandidateIndexes<Base: SudokuBase> {
    rows: CandidatesGroup<Base>,
    columns: CandidatesGroup<Base>,
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

                    let row_index = pos.to_column();
                    group_candidate_indexes
                        .rows
                        .get_mut(pos.to_row())
                        .insert(row_index);
                    let column_index = pos.to_row();
                    group_candidate_indexes
                        .columns
                        .get_mut(pos.to_column())
                        .insert(column_index);
                }
            }
        }
        candidate_to_group_candidate_indexes
    }

    fn axis(&self, axis: Axis) -> &CandidatesGroup<Base> {
        match axis {
            Axis::Row => &self.rows,
            Axis::Column => &self.columns,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Axis {
    Row,
    Column,
}

impl Axis {
    fn other(self) -> Self {
        match self {
            Axis::Row => Axis::Column,
            Axis::Column => Axis::Row,
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
        base::consts::*, cell::Cell, solver::strategic::strategies::test_util::assert_deductions,
    };
    use indoc::indoc;

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

        let candidate = Value::try_from(1).unwrap();
        let action = Action::delete_candidate(candidate);
        let reason = Reason::candidate(candidate);
        let expected_deductions = std::iter::once(
            Deduction::try_from_iters(
                vec![
                    //
                    ((0, 1), action),
                    ((3, 3), action),
                ],
                vec![
                    ((1, 1), reason),
                    ((1, 3), reason),
                    ((2, 1), reason),
                    ((2, 3), reason),
                ],
            )
            .unwrap(),
        )
        .collect();

        assert_deductions(&deductions, &expected_deductions);
    }

    #[test]
    fn test_synthetic_base_2_column_to_row() {
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

        let candidate = Value::try_from(1).unwrap();
        let action = Action::delete_candidate(candidate);
        let reason = Reason::candidate(candidate);
        let expected_deductions = std::iter::once(
            Deduction::try_from_iters(
                vec![
                    //
                    ((1, 0), action),
                    ((2, 2), action),
                ],
                vec![
                    ((1, 1), reason),
                    ((1, 3), reason),
                    ((2, 1), reason),
                    ((2, 3), reason),
                ],
            )
            .unwrap(),
        )
        .collect();

        assert_deductions(&deductions, &expected_deductions);
    }
}
