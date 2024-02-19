use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::samples;
use sudoku::solver::sat::Solver;

fn main() -> Result<()> {
    type Base = Base3;

    for (i, grid) in samples::base_3().into_iter().enumerate() {
        println!("{grid}");
        let grid_assignments = Solver::<Base>::grid_assignments(&grid);
        println!("{grid_assignments:?}");

        let path = format!("./sudoku-rs/out/grid_assignments/base3_sample_{i}.json");
        let writer = BufWriter::new(File::create(Path::new(&path))?);
        serde_json::to_writer_pretty(writer, &grid_assignments)?;
    }

    Ok(())
}
