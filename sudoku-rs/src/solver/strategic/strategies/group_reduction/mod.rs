use itertools::izip;

use crate::cell::SudokuCell;
use crate::grid::Grid;
use crate::position::Position;

use super::Strategy;

use self::pcp::group_candidates_reduction;

mod pcp;

#[derive(Debug)]
pub struct GroupReduction;

impl<Cell: SudokuCell> Strategy<Cell> for GroupReduction {
    fn execute(&self, grid: &mut Grid<Cell>) -> Vec<Position> {
        let mut modified_positions = vec![];

        Self::reduce_groups(grid.all_row_positions(), grid, &mut modified_positions);
        Self::reduce_groups(grid.all_column_positions(), grid, &mut modified_positions);
        Self::reduce_groups(grid.all_block_positions(), grid, &mut modified_positions);

        modified_positions.sort();
        modified_positions.dedup();
        modified_positions
    }
}

impl GroupReduction {
    fn reduce_groups<Cell: SudokuCell>(
        groups: impl Iterator<Item = impl Iterator<Item = Position>>,
        grid: &mut Grid<Cell>,
        modified_positions: &mut Vec<Position>,
    ) {
        for group in groups {
            let (positions, group_candidates): (Vec<Position>, Vec<Vec<usize>>) = group
                .filter_map(|pos| {
                    let cell = grid.get(pos);
                    cell.candidates().map(|candidates| (pos, candidates))
                })
                .unzip();

            let reduced_group_candidates =
                group_candidates_reduction(&group_candidates, grid.max_value());

            for zipped in izip!(
                positions.clone(),
                group_candidates.clone(),
                reduced_group_candidates.clone()
            ) {
                let (pos, candidates, reduced_candidates): (Position, Vec<usize>, Vec<usize>) =
                    zipped;

                if candidates != reduced_candidates {
                    println!(
                        "GroupReduction at {}: {:?} => {:?}",
                        pos, candidates, reduced_candidates
                    );

                    //                    eprintln!(
                    //                        "{:?}, {:?}, {:?}",
                    //                        positions, group_candidates, reduced_group_candidates
                    //                    );

                    grid.set_candidates(pos, reduced_candidates);

                    modified_positions.push(pos)
                }
            }
        }
    }
}
