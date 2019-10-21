use itertools::izip;

use crate::base::SudokuBase;
use crate::cell::SudokuCell;
use crate::grid::Grid;
use crate::position::Position;

use super::Strategy;

use self::pcp::group_candidates_reduction;

mod pcp;

#[derive(Debug)]
pub struct GroupReduction;

impl<Base: SudokuBase> Strategy<Base> for GroupReduction {
    fn execute(&self, grid: &mut Grid<Base>) -> Vec<Position> {
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
    fn reduce_groups<Base: SudokuBase>(
        groups: impl Iterator<Item = impl Iterator<Item = Position>>,
        grid: &mut Grid<Base>,
        modified_positions: &mut Vec<Position>,
    ) {
        for group in groups {
            let (positions, group_candidates): (Vec<Position>, Vec<Vec<u8>>) = group
                .filter_map(|pos| {
                    let cell = grid.get(pos);
                    cell.candidates().map(|candidates| (pos, candidates))
                })
                .unzip();

            let reduced_group_candidates =
                group_candidates_reduction(&group_candidates, Grid::<Base>::max_value());

            for zipped in izip!(
                positions.clone(),
                group_candidates.clone(),
                reduced_group_candidates.clone()
            ) {
                let (pos, candidates, reduced_candidates): (Position, Vec<u8>, Vec<u8>) = zipped;

                if candidates != reduced_candidates {
                    println!(
                        "GroupReduction at {}: {:?} => {:?}",
                        pos, candidates, reduced_candidates
                    );

                    //                    eprintln!(
                    //                        "{:?}, {:?}, {:?}",
                    //                        positions, group_candidates, reduced_group_candidates
                    //                    );

                    grid.get_mut(pos).set_candidates(reduced_candidates);

                    modified_positions.push(pos)
                }
            }
        }
    }
}
