use sudoku::cell::OptionCell;
use sudoku::generator::RandomGenerator;

fn main() {
    let generator = RandomGenerator::new(2, 1_000, false);
    let sudoku = generator.generate::<OptionCell>().unwrap();

    println!("{}", sudoku);
}
