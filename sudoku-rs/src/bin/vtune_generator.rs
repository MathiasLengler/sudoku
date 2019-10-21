#[cfg(feature = "flame_it")]
extern crate flame;

#[cfg(feature = "flame_it")]
use std::fs::File;

use typenum::U2;

use sudoku::error::Result;
use sudoku::generator::backtracking::{Generator, Target};

// add in lib
// #[cfg_attr(feature = "flame_it", flame)]

fn main() -> Result<()> {
    #[cfg(feature = "flame_it")]
    println!("Flame it enabled");

    for i in 0..1 {
        dbg!(i);
        Generator::with_target(Target::Minimal)
            .generate::<U2>()
            .unwrap();
    }

    #[cfg(feature = "flame_it")]
    std::fs::create_dir_all("target/flame")?;

    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut File::create("target/flame/flame-graph.html")?)?;

    Ok(())
}
