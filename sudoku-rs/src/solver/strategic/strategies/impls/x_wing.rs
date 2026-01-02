use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
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

    fn execute<Base: SudokuBase>(self, _grid: &Grid<Base>) -> Result<Deductions<Base>> {
        todo!("X-Wing strategy not yet implemented")
    }
}
