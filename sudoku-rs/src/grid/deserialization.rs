use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;

pub fn read_grids_from_file<Base: SudokuBase>(path: impl AsRef<Path>) -> Result<Vec<Grid<Base>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .filter_map(|line| {
            let line = line.unwrap();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let grid: Grid<_> = line.as_str().try_into().unwrap();
            Some(grid)
        })
        .collect())
}
