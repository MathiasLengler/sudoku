use hdrhistogram::Histogram;

use sudoku::base::consts::Base3;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorTarget};
use sudoku::grid::Grid;
use sudoku::solver::backtracking_bitset::Solver;

type Base = Base3;

fn main() -> Result<()> {
    let mut hist = Histogram::<u64>::new_with_bounds(1, 10u64.pow(6), 4).unwrap();

    let corners_2x2 = vec![
        // top right
        vec![0, 1, 3, 4],
        // top left
        vec![1, 2, 4, 5],
        // bottom left
        vec![3, 4, 6, 7],
        // bottom right
        vec![4, 5, 7, 8],
    ];

    for _ in 0..1000 {
        let filled_grid = Generator::<Base>::with_target(GeneratorTarget::Filled)
            .generate()
            .unwrap();
        let mut grid = filled_grid.clone();

        grid.unfix_all_values();

        corners_2x2.iter().for_each(|corner_2x2| {
            let mut grid = filled_grid.clone();
            grid.unfix_all_values();

            for (i, column_positions) in Grid::all_block_positions().enumerate() {
                if corner_2x2.contains(&i) {
                    for column_position in column_positions {
                        grid[column_position].delete();
                    }
                }
            }
            let solution_count = u64::try_from(Solver::new(&grid).count()).unwrap();
            // dbg!(solution_count);
            hist += solution_count;
        });
    }

    println!("# of samples for solution count: {}", hist.len());
    dbg!(hist.min());
    dbg!(hist.max());

    for v in hist.iter_recorded() {
        println!(
            "{}'th percentile of data is {} with {} samples",
            v.percentile(),
            v.value_iterated_to(),
            v.count_at_value()
        );
    }
    println!("99.9'th percentile: {}", hist.value_at_quantile(0.999));

    Ok(())
}
