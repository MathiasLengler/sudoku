use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::samples;
use sudoku::solver::sat::Solver;
use varisat::Lit;

fn main() -> Result<()> {
    type Base = Base3;

    let path = Path::new(r"D:\git\personal\pysat-playground\out\assignment.json");

    let assignment: Vec<i32> = serde_json::from_reader(BufReader::new(File::open(path)?))?;
    let solution = Solver::<Base>::assigment_to_solution(
        &assignment
            .into_iter()
            .map(|i| Lit::from_dimacs(i.try_into().unwrap()))
            .collect::<Vec<_>>(),
    )?;
    println!("Solution:\n{solution}");

    let puzzle = samples::base_3().into_iter().last().unwrap();
    println!("Puzzle:\n{puzzle}");
    solution.assert_is_solution_for(&puzzle);

    Ok(())
}
