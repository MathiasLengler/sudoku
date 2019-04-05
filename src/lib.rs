use std::fmt::{Display, Formatter};
use std::fmt;

use cell::SudokuCell;

use crate::position::Position;

pub mod cell;
pub mod position;


// TODO: generate valid solved sudoku
// TODO: generate valid incomplete sudoku
// TODO: solve/verify incomplete sudoku

#[derive(Clone, Debug)]
pub struct Sudoku<Cell: SudokuCell> {
    base: usize,
    cells: Vec<Cell>,

}

// TODO: rethink indexing story (internal/cell position/block position)
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub fn new(base: usize) -> Self {
        let mut sudoku = Sudoku {
            base,
            cells: vec![],
        };

        sudoku.cells = vec![Default::default(); sudoku.cell_count()];
        sudoku
    }

    pub fn has_conflict(&self) -> bool {
        self.all_rows().any(|row| self.has_duplicate(row)) ||
        self.all_columns().any(|column| self.has_duplicate(column)) ||
        self.all_blocks().any(|block| self.has_duplicate(block))
    }

    // TODO: conflict location pairs
    fn has_duplicate<'a>(&'a self, cells: impl Iterator<Item=&'a Cell>) -> bool {
        let mut cells: Vec<_> = cells.filter(|cell| cell.has_value()).collect();

        cells.sort();

        let cell_count = cells.len();

        cells.dedup();

        let cell_count_dedup = cells.len();

        cell_count != cell_count_dedup
    }

    // TODO: Cell with Position
    // TODO: rethink parameter
    fn row(&self, pos: Position) -> impl Iterator<Item=&Cell> {
        self.assert_position(pos);

        let starting_index = pos.y * self.side_length();

        (starting_index..starting_index + self.side_length()).map(move |i| &self.cells[i])
    }

    fn all_rows(&self) -> impl Iterator<Item=impl Iterator<Item=&Cell>> {
        (0..self.side_length()).map(move |row_index| {
            self.row(Position {
                x: 0,
                y: row_index,
            })
        })
    }


    // TODO: Cell with Position
    // TODO: rethink parameter
    fn column(&self, pos: Position) -> impl Iterator<Item=&Cell> {
        self.assert_position(pos);

        let starting_index = pos.x;

        (starting_index..self.cell_count()).step_by(self.side_length()).map(move |i| &self.cells[i])
    }

    fn all_columns(&self) -> impl Iterator<Item=impl Iterator<Item=&Cell>> {
        (0..self.side_length()).map(move |row_index| {
            self.column(Position {
                x: row_index,
                y: 0,
            })
        })
    }

    // TODO: Cell with Position
    // TODO: rethink parameter
    fn block(&self, pos: Position) -> impl Iterator<Item=&Cell> {
        self.assert_position(pos);

        let block_base_pos = (pos / self.base) * self.base;

        let block_base_index = self.index_at(block_base_pos);

        (block_base_index..self.cell_count())
            .step_by(self.side_length())
            .take(self.base)
            .flat_map(move |block_row_start_index| {
                (block_row_start_index..block_row_start_index + self.base)
            })
            .map(move |i| &self.cells[i])
    }

    fn all_blocks(&self) -> impl Iterator<Item=impl Iterator<Item=&Cell>> {
        let all_block_base_pos =
            (0..self.base)
                .flat_map(
                    move |block_y| (0..self.base).map(move |block_x| Position {
                        x: block_x,
                        y: block_y,
                    })
                )
                .map(move |pos| pos * self.base);

        all_block_base_pos.map(move |block_base_pos| self.block(block_base_pos))
    }


    pub fn get(&self, pos: Position) -> &Cell {
        self.assert_position(pos);

        let index = self.index_at(pos);

        &self.cells[index]
    }

    pub fn set(&mut self, pos: Position, value: Cell) {
        self.assert_position(pos);

        let index = self.index_at(pos);

        self.cells[index] = value;
    }

    fn assert_position(&self, pos: Position) {
        assert!(pos.x < self.side_length() && pos.y < self.side_length())
    }

    fn index_at(&self, pos: Position) -> usize {
        pos.x + pos.y * self.side_length()
    }

//    fn pos_at(&self, index: usize) -> Position {
//        Position {
//            x: index / self.side_length(),
//            y: index % self.side_length(),
//        }
//    }

    fn side_length(&self) -> usize {
        self.base.pow(2)
    }

    fn cell_count(&self) -> usize {
        self.base.pow(4)
    }
}

impl<Cell: SudokuCell> Display for Sudoku<Cell> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(
            &self.cells
                .chunks(self.side_length())
                .map(|chunk| chunk.iter().map(ToString::to_string).collect::<String>())
                .collect::<Vec<_>>().join("\n")
        )
    }
}

