use ndarray::{self, Array2, s};
use sudoku::error::Result;

fn main() -> Result<()> {
    let vec: Vec<_> = (0..81).collect();

    let arr = Array2::from_shape_vec((9,9), vec)?;

    dbg!(&arr);
    println!("{}", arr);
    println!("{}", arr.slice(s![0..3, 0..3]));
    println!("{}", arr.row(2));
    println!("{}", arr.column(2));

    Ok(())
}