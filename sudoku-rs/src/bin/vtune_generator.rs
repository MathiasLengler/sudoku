#[cfg(feature = "flame_it")]
extern crate flame;

use std::fs::File;

use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::generator::backtracking::{Generator, Settings, Target};

// add in lib
// #[cfg_attr(feature = "flame_it", flame)]

fn main() -> Result<()> {
    #[cfg(feature = "flame_it")]
    println!("Flame it enabled");

    for i in 0..1 {
        dbg!(i);
        Generator::new(Settings {
            base: 2,
            target: Target::Minimal,
        })
        .generate::<Cell>()
        .unwrap();
    }

    #[cfg(feature = "flame_it")]
    std::fs::create_dir_all("target/flame")?;

    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut File::create("target/flame/flame-graph.html")?)?;

    Ok(())
}
