use sudoku::cell::OptionCell;
use sudoku::generator::ParallelRandomGenerator;

fn main() {
    let generator = ParallelRandomGenerator::new(1_000_000);
    let sudoku = generator.generate::<OptionCell>().unwrap();

    println!("{}", sudoku);
}
