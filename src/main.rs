use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use rayon::prelude::ParallelIterator;

use sudoku::cell::OptionCell;
use sudoku::position::Position;
use sudoku::Sudoku;

fn main() {
    let complete_sudoku: Option<Option<Sudoku<OptionCell>>> = (0..1_000_000)
        .into_par_iter()
        .map(|_try_count| {
            let mut sudoku = Sudoku::<OptionCell>::new(3);

//            if try_count % 100 == 0 {
//                eprintln!("try_count = {:?}", try_count);
//            }

            if try_fill(&mut sudoku) {
                Some(sudoku)
            } else {
                None
            }
        })
        .find_any(Option::is_some);

    eprintln!("complete_sudoku = \n{}", complete_sudoku.unwrap().unwrap());
}

fn try_fill(sudoku: &mut Sudoku<OptionCell>) -> bool {
    let mut positions: Vec<_> = sudoku.all_positions().collect();

    let mut rng = thread_rng();

    positions.shuffle(&mut rng);

    let mut no_deadlock = true;

    'outer: for pos in positions {
        for value in 1..=sudoku.side_length() {
            sudoku.set(pos, OptionCell(Some(value)));

            if !sudoku.has_conflict() {
                continue 'outer;
            }
        }
        no_deadlock = false;

        break;
    }

    no_deadlock
}

#[allow(dead_code)]
fn debug() {
    let mut sudoku = Sudoku::<OptionCell>::new(3);

    let mut debug_value = 0;
    for y in 0..sudoku.side_length() {
        for x in 0..sudoku.side_length() {
            sudoku.set(Position { x, y }, OptionCell(Some(debug_value)));
            debug_value += 1;
        }
    }

    println!("{}", sudoku);
    dbg!(sudoku.has_conflict());

    sudoku.set(Position {
        x: 2,
        y: 2,
    }, OptionCell(Some(0)));

    println!("{}", sudoku);
    dbg!(sudoku.has_conflict());
}
