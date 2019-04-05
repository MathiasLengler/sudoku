use core::num::FpCategory::Subnormal;
use std::fmt::{Display, Formatter};
use std::fmt;

fn main() {
    let mut sudoku = Sudoku::<u8>::new(3);

    sudoku.set(Position {
        x: 8,
        y: 0
    }, 1);

    eprintln!("sudoku = {:#?}", sudoku);
    println!("sudoku = {}", sudoku);


}

#[derive(Clone, Debug)]
struct Sudoku<Cell: Sized + Default + Clone + Display> {
    base: u8,
    array: Vec<Option<Cell>>,
}

impl<Cell: Sized + Default + Clone + Display> Display for Sudoku<Cell> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(
            &self.array
                .chunks(Self::base_to_side_length(self.base))
                .map(|chunk| chunk.iter().map(|cell| match cell {
                    Some(cell) => cell.to_string(),
                    None => " ".to_string()
                }).collect::<String>())
                .collect::<Vec<_>>().join("\n")
        )
    }
}

struct Position {
    pub x: usize,
    pub y: usize,
}

impl<Cell: Sized + Default + Clone + Display> Sudoku<Cell> {
    pub fn new(base: u8) -> Self {
        Sudoku {
            base,
            array: vec![Default::default(); Self::base_to_cell_count(base)],
        }
    }

    pub fn set(&mut self, at: Position, value: Cell) {
        self.assert_position(at)

        unimplemented!()
    }

    fn assert_position(&self, pos: Position) {
        assert!(pos.x < self.side_length() && pos.y < self.side_length())
    }

    fn side_length(&self) -> usize {
        Self::base_to_side_length(self.base)
    }

    fn base_to_side_length(base: u8) -> usize {
        (base as usize).pow(2)
    }

    fn cell_count(&self) -> usize {
        Self::base_to_cell_count(self.base)
    }

    fn base_to_cell_count(base: u8) -> usize {
        (base as usize).pow(3)
    }
}

