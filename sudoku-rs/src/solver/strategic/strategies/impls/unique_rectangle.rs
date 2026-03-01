use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::{Coordinate, Position};
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};
use crate::solver::strategic::strategies::{Strategy, StrategyScore};

/// Unique Rectangle (Type 1) elimination strategy.
///
/// A Unique Rectangle occurs when four cells at the corners of a rectangle
/// (exactly two rows, two columns, spanning exactly two blocks) all contain
/// candidates {a, b}. Since a valid sudoku must have a unique solution,
/// this "deadly pattern" cannot occur. If three corners contain only {a, b}
/// and the fourth contains {a, b} plus extra candidates, we can eliminate
/// {a, b} from that fourth corner.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UniqueRectangle;

impl Strategy for UniqueRectangle {
    fn name(self) -> &'static str {
        "UniqueRectangle"
    }

    fn score(self) -> StrategyScore {
        100
    }

    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(find_unique_rectangles(grid))
    }
}

fn find_unique_rectangles<Base: SudokuBase>(grid: &Grid<Base>) -> Deductions<Base> {
    let candidates_all = Candidates::<Base>::all();

    // For each pair of candidate values (a, b), find unique rectangle patterns
    candidates_all
        .into_iter()
        .flat_map(|a| {
            candidates_all
                .into_iter()
                .filter(move |&b| a < b)
                .map(move |b| Candidates::with_single(a).union(Candidates::with_single(b)))
        })
        .flat_map(|pair| find_unique_rectangles_for_pair(grid, pair))
        .collect()
}

fn find_unique_rectangles_for_pair<Base: SudokuBase>(
    grid: &Grid<Base>,
    pair: Candidates<Base>,
) -> Vec<Deduction<Base>> {
    let side_length = usize::from(Base::SIDE_LENGTH);

    // For each row, collect column coordinates where the cell contains both candidates of the pair
    let mut rows_with_pair_columns: Vec<(Coordinate<Base>, Vec<Coordinate<Base>>)> =
        Vec::with_capacity(side_length);

    for row in Coordinate::<Base>::all() {
        let columns: Vec<_> = Coordinate::<Base>::all()
            .filter(|&col| {
                let pos = Position::from((row, col));
                grid[pos]
                    .candidates()
                    .is_some_and(|c| c.intersection(pair) == pair)
            })
            .collect();

        if columns.len() >= 2 {
            rows_with_pair_columns.push((row, columns));
        }
    }

    let mut deductions = Vec::new();

    // For each pair of rows
    for i in 0..rows_with_pair_columns.len() {
        for j in (i + 1)..rows_with_pair_columns.len() {
            let (r1, columns_a) = &rows_with_pair_columns[i];
            let (r2, columns_b) = &rows_with_pair_columns[j];

            // Find common columns (present in both rows)
            for ci in 0..columns_a.len() {
                for cj in (ci + 1)..columns_a.len() {
                    let c1 = columns_a[ci];
                    let c2 = columns_a[cj];

                    // Check if both columns are also present in row2
                    if !columns_b.contains(&c1) || !columns_b.contains(&c2) {
                        continue;
                    }

                    // We have a rectangle with corners
                    let corners = [
                        Position::from((*r1, c1)),
                        Position::from((*r1, c2)),
                        Position::from((*r2, c1)),
                        Position::from((*r2, c2)),
                    ];

                    // Check that the rectangle spans exactly 2 blocks
                    let blocks: std::collections::BTreeSet<_> =
                        corners.iter().map(|p| p.to_block()).collect();
                    if blocks.len() != 2 {
                        continue;
                    }

                    // Try Type 1: exactly 3 floor cells (candidates == pair) and 1 roof cell
                    if let Some(deduction) = try_type1(grid, pair, corners) {
                        deductions.push(deduction);
                    }

                    // Try Type 2: exactly 2 floor cells and 2 roof cells with same extra candidate
                    deductions.extend(try_type2(grid, pair, corners));
                }
            }
        }
    }

    deductions
}

/// Type 1: Three floor cells have exactly {a, b}, one roof cell has {a, b} + extras.
/// Eliminate {a, b} from the roof cell.
fn try_type1<Base: SudokuBase>(
    grid: &Grid<Base>,
    pair: Candidates<Base>,
    corners: [Position<Base>; 4],
) -> Option<Deduction<Base>> {
    let mut floor_positions = Vec::with_capacity(3);
    let mut roof_position = None;

    for pos in corners {
        let candidates = grid[pos].candidates()?;
        if candidates == pair {
            floor_positions.push(pos);
        } else if candidates.intersection(pair) == pair {
            // Has pair plus extras
            if roof_position.is_some() {
                return None; // More than one roof cell → not Type 1
            }
            roof_position = Some(pos);
        } else {
            return None; // Cell doesn't contain the full pair
        }
    }

    let roof_pos = roof_position?;
    if floor_positions.len() != 3 {
        return None;
    }

    // Delete the pair candidates from the roof cell
    Some(
        Deduction::try_from_iters(
            std::iter::once((roof_pos, Action::delete_candidates(pair))),
            floor_positions
                .into_iter()
                .map(|pos| (pos, Reason::candidates(pair))),
        )
        .unwrap(),
    )
}

/// Type 2: Two floor cells have exactly {a, b}, two roof cells (in the same row or column)
/// have {a, b, x} for the same extra candidate x.
/// Eliminate x from cells that see both roof cells.
fn try_type2<Base: SudokuBase>(
    grid: &Grid<Base>,
    pair: Candidates<Base>,
    corners: [Position<Base>; 4],
) -> Option<Deduction<Base>> {
    let mut floor_positions = Vec::with_capacity(2);
    let mut roof_positions = Vec::with_capacity(2);

    for pos in corners {
        let candidates = grid[pos].candidates()?;
        if candidates == pair {
            floor_positions.push(pos);
        } else if candidates.intersection(pair) == pair {
            roof_positions.push((pos, candidates));
        } else {
            return None;
        }
    }

    if floor_positions.len() != 2 || roof_positions.len() != 2 {
        return None;
    }

    let (roof1_pos, roof1_candidates) = roof_positions[0];
    let (roof2_pos, roof2_candidates) = roof_positions[1];

    // Both roof cells must have exactly one extra candidate, and it must be the same
    let roof1_extras = roof1_candidates.without(pair);
    let roof2_extras = roof2_candidates.without(pair);

    if roof1_extras.count() != 1 || roof1_extras != roof2_extras {
        return None;
    }

    let extra = roof1_extras;

    // Roof cells must be in the same row or same column
    let in_same_row = roof1_pos.to_row() == roof2_pos.to_row();
    let in_same_column = roof1_pos.to_column() == roof2_pos.to_column();

    if !in_same_row && !in_same_column {
        return None;
    }

    // Find cells that see both roof cells and contain the extra candidate
    let actions: Vec<_> = Position::<Base>::all()
        .filter(|&pos| pos != roof1_pos && pos != roof2_pos)
        .filter(|&pos| {
            grid[pos]
                .candidates()
                .is_some_and(|c| !c.intersection(extra).is_empty())
        })
        .filter(|&pos| shares_group(pos, roof1_pos) && shares_group(pos, roof2_pos))
        .map(|pos| (pos, Action::delete_candidates(extra)))
        .collect();

    if actions.is_empty() {
        return None;
    }

    // All 4 corners form the deadly pattern and are included as reasons.
    // Roof cells highlight the pair + extra, floor cells highlight just the pair.
    let reasons: Vec<_> = roof_positions
        .iter()
        .map(|&(pos, _)| (pos, Reason::candidates(pair.union(extra))))
        .chain(
            floor_positions
                .iter()
                .map(|&pos| (pos, Reason::candidates(pair))),
        )
        .collect();

    Deduction::try_from_iters(actions, reasons).ok()
}

/// Check if two positions share any group (row, column, or block).
fn shares_group<Base: SudokuBase>(a: Position<Base>, b: Position<Base>) -> bool {
    a.to_row() == b.to_row() || a.to_column() == b.to_column() || a.to_block() == b.to_block()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::consts::*,
        cell::{Cell, Value},
        solver::strategic::strategies::test_util::{
            assert_deductions, strategy_snapshot_tests,
        },
    };

    fn ur_deduction<Base: SudokuBase>(
        positions_to_delete: impl IntoIterator<Item = ((u8, u8), Vec<u8>)>,
        reason_positions: impl IntoIterator<Item = ((u8, u8), Vec<u8>)>,
    ) -> Deduction<Base> {
        Deduction::try_from_iters(
            positions_to_delete.into_iter().map(|(pos, candidates)| {
                let cands: Candidates<Base> = candidates
                    .into_iter()
                    .map(|c| c.try_into().unwrap())
                    .fold(Candidates::new(), |acc, v: Value<Base>| {
                        acc.union(Candidates::with_single(v))
                    });
                (pos, Action::delete_candidates(cands))
            }),
            reason_positions.into_iter().map(|(pos, candidates)| {
                let cands: Candidates<Base> = candidates
                    .into_iter()
                    .map(|c| c.try_into().unwrap())
                    .fold(Candidates::new(), |acc, v: Value<Base>| {
                        acc.union(Candidates::with_single(v))
                    });
                (pos, Reason::candidates(cands))
            }),
        )
        .unwrap()
    }

    mod type1 {
        use super::*;

        #[test]
        fn test_type1_base2() {
            // Set up a 4x4 grid where:
            // - (0,0), (0,2), (1,0) have exactly candidates {1,2}
            // - (1,2) has candidates {1,2,3}
            // The rectangle spans blocks 0 and 1 (top-left and top-right)
            // Expected: eliminate {1,2} from (1,2)
            let mut grid: Grid<Base2> = Grid::default();

            let pair = Candidates::with_single(1u8.try_into().unwrap())
                .union(Candidates::with_single(2u8.try_into().unwrap()));
            let roof_cands = pair.union(Candidates::with_single(3u8.try_into().unwrap()));

            // Set floor cells
            grid[Position::from((
                Coordinate::try_from(0u8).unwrap(),
                Coordinate::try_from(0u8).unwrap(),
            ))] = Cell::with_candidates(pair);
            grid[Position::from((
                Coordinate::try_from(0u8).unwrap(),
                Coordinate::try_from(2u8).unwrap(),
            ))] = Cell::with_candidates(pair);
            grid[Position::from((
                Coordinate::try_from(1u8).unwrap(),
                Coordinate::try_from(0u8).unwrap(),
            ))] = Cell::with_candidates(pair);

            // Set roof cell
            grid[Position::from((
                Coordinate::try_from(1u8).unwrap(),
                Coordinate::try_from(2u8).unwrap(),
            ))] = Cell::with_candidates(roof_cands);

            // Set other cells to have some value so they don't interfere
            for pos in Position::<Base2>::all() {
                let (row, col) = pos.to_row_and_column();
                let is_corner = (row.get() == 0 || row.get() == 1)
                    && (col.get() == 0 || col.get() == 2);
                if !is_corner {
                    grid[pos] = Cell::with_candidates(Candidates::with_single(
                        4u8.try_into().unwrap(),
                    ));
                }
            }

            let deductions = UniqueRectangle.execute(&grid).unwrap();

            let expected_deductions: Deductions<Base2> = ur_deduction(
                vec![((1, 2), vec![1, 2])],
                vec![
                    ((0, 0), vec![1, 2]),
                    ((0, 2), vec![1, 2]),
                    ((1, 0), vec![1, 2]),
                ],
            )
            .into();

            assert_deductions(&deductions, &expected_deductions);
        }
    }

    strategy_snapshot_tests!(UniqueRectangle);
}
