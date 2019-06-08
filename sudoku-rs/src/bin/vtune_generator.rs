use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::generator::backtracking::{Generator, Settings, Target};

fn main() -> Result<()> {
    for i in 0..20 {
        dbg!(i);
        Generator::new(Settings {
            base: 3,
            target: Target::Minimal,
        })
        .generate::<Cell>()
        .unwrap();
    }

    Ok(())
}
