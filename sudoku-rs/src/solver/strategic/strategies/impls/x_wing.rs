use std::collections::BTreeMap;

use itertools::izip;

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::group::CandidatesGroup;
use crate::grid::Grid;
use crate::position::Coordinate;
use crate::position::Position;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;

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

        Ok(
            izip!(Value::<Base>::all(), candidate_to_group_candidate_indexes)
                .flat_map(|(candidate, group_candidate_indexes)| {
                    // row to column eliminations

                    let locked_pairs: Vec<_> = group_candidate_indexes
                        .rows
                        .iter_enumerate()
                        .filter(|(_, candidates)| candidates.count() == 2)
                        .collect();

                    let mut locked_pair_pattern_to_row_coordinates: BTreeMap<
                        Candidates<_>,
                        Vec<Coordinate<_>>,
                    > = BTreeMap::new();

                    for (row_coordinate, locked_pair_pattern) in locked_pairs {
                        locked_pair_pattern_to_row_coordinates
                            .entry(locked_pair_pattern)
                            .and_modify(|row_coordinates| row_coordinates.push(row_coordinate))
                            .or_insert(vec![row_coordinate]);
                    }

                    let x_wing_candidates: Vec<XWingPattern<Base>> =
                        locked_pair_pattern_to_row_coordinates
                            .into_iter()
                            .filter_map(|(locked_pair_pattern, row_coordinates)| {
                                    // 
                                match row_coordinates.as_slice() {
                                    [] => {
                                        panic!(
                                            "Expected at least one coordinate for pattern {locked_pair_pattern}",
                                        )
                                    },
                                    [_] => None,
                                    [first_row_coordinate, second_row_coordinate] => {
                                        // X-Wing candidate found
                                        Some(XWingPattern {
                                            candidate,
                                            row_coordinates: todo!(),
                                            column_coordinates: todo!(),
                                        })
                                    },
                                    conflicting_row_coordinates => {
                                            // TODO: return an Error instead of panicking
                                        panic!(
                                            "Sudoku is unsolvable: conflicting row coordinates {conflicting_row_coordinates:?} for locked pair pattern {locked_pair_pattern}"
                                        )
                                    }
                                }
                            })
                            .collect();

                    vec![]
                })
                .collect(),
        )
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct XWingPattern<Base: SudokuBase> {
    candidate: Value<Base>,
    row_coordinates: [Coordinate<Base>; 2],
    column_coordinates: [Coordinate<Base>; 2],
}
